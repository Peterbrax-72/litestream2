(()=>{
  const desc=(node,name)=>node?Array.from(node.getElementsByTagNameNS('*',name)):[];
  const first=(node,name)=>desc(node,name)[0]||null;
  function color(node,fallback){
    if(!node)return fallback;
    const rgb=first(node,'srgbClr')?.getAttribute('val');
    if(rgb&&/^[0-9a-f]{6}$/i.test(rgb))return '#'+rgb;
    const scheme=first(node,'schemeClr')?.getAttribute('val');
    return ({lt1:'#ffffff',dk1:'#111111',lt2:'#eeeeee',dk2:'#333333',accent1:'#4472c4',accent2:'#ed7d31',accent3:'#a5a5a5',accent4:'#ffc000',accent5:'#5b9bd5',accent6:'#70ad47',hlink:'#0563c1'})[scheme]||fallback;
  }
  function resolvePath(base,target){
    const clean=decodeURIComponent(target||'').replace(/\\/g,'/');
    if(clean.startsWith('/'))return clean.slice(1);
    const parts=base.split('/').slice(0,-1);
    for(const bit of clean.split('/')){if(!bit||bit==='.')continue;if(bit==='..')parts.pop();else parts.push(bit)}
    return parts.join('/');
  }
  async function unzip(buffer){
    const bytes=new Uint8Array(buffer),view=new DataView(buffer);let end=-1;
    for(let i=bytes.length-22;i>=Math.max(0,bytes.length-65558);i--){if(view.getUint32(i,true)===0x06054b50){end=i;break}}
    if(end<0)throw new Error('This is not a supported PowerPoint ZIP package');
    const total=view.getUint16(end+10,true),central=view.getUint32(end+16,true);
    if(total>12000)throw new Error('Presentation package contains too many files');
    const entries=new Map();let cursor=central,expandedTotal=0;
    for(let n=0;n<total;n++){
      if(view.getUint32(cursor,true)!==0x02014b50)throw new Error('Invalid PowerPoint package directory');
      const method=view.getUint16(cursor+10,true),compressed=view.getUint32(cursor+20,true),uncompressed=view.getUint32(cursor+24,true),nameLen=view.getUint16(cursor+28,true),extraLen=view.getUint16(cursor+30,true),commentLen=view.getUint16(cursor+32,true),local=view.getUint32(cursor+42,true),name=new TextDecoder().decode(bytes.slice(cursor+46,cursor+46+nameLen));
      expandedTotal+=uncompressed;if(expandedTotal>240*1024*1024)throw new Error('Expanded PowerPoint content exceeds the 240 MB safety limit');entries.set(name,{method,compressed,uncompressed,local});cursor+=46+nameLen+extraLen+commentLen;
    }
    async function read(name){
      const entry=entries.get(name);if(!entry)return null;if(entry.uncompressed>64*1024*1024)throw new Error('A PowerPoint asset exceeds the 64 MB safety limit');
      const at=entry.local;if(view.getUint32(at,true)!==0x04034b50)throw new Error('Corrupt PPTX entry');
      const start=at+30+view.getUint16(at+26,true)+view.getUint16(at+28,true),packed=bytes.slice(start,start+entry.compressed);
      if(entry.method===0)return packed;
      if(entry.method===8){if(typeof DecompressionStream==='undefined')throw new Error('This runtime cannot decompress PPTX files');const stream=new Blob([packed]).stream().pipeThrough(new DecompressionStream('deflate-raw'));return new Uint8Array(await new Response(stream).arrayBuffer())}
      throw new Error('Unsupported PPTX compression method '+entry.method);
    }
    return {entries,read};
  }
  function boxFor(node,slideW,slideH){
    const xfrm=first(node,'xfrm'),off=first(xfrm,'off'),ext=first(xfrm,'ext');if(!off||!ext)return null;
    return {x:Number(off.getAttribute('x')||0)/slideW,y:Number(off.getAttribute('y')||0)/slideH,w:Number(ext.getAttribute('cx')||0)/slideW,h:Number(ext.getAttribute('cy')||0)/slideH};
  }
  function drawSlide(elements,background){
    const canvas=document.createElement('canvas');canvas.width=960;canvas.height=540;const c=canvas.getContext('2d');
    c.fillStyle=background||'#ffffff';c.fillRect(0,0,canvas.width,canvas.height);
    for(const item of elements){
      const x=item.x*canvas.width,y=item.y*canvas.height,w=item.w*canvas.width,h=item.h*canvas.height;
      if(item.kind==='image'&&item.image){c.drawImage(item.image,x,y,w,h);continue}
      if(item.fill){c.fillStyle=item.fill;c.fillRect(x,y,w,h)}
      if(item.text){c.save();const fontSize=Math.max(12,Math.round(item.fontSize||24)),style=item.italic?'italic ':'',weight=item.bold?'700':'400';c.font=`${style}${weight} ${fontSize}px ${item.fontFamily||'Arial'}`;c.fillStyle=item.textColor||'#161616';c.textBaseline='top';c.textAlign=item.align||'left';const pad=Math.max(5,fontSize*.18),lineH=fontSize*1.2;item.text.split('\n').slice(0,12).forEach((line,i)=>{const tx=item.align==='center'?x+w/2:item.align==='right'?x+w-pad:x+pad;c.fillText(line,tx,y+pad+i*lineH,Math.max(20,w-pad*2))});c.restore()}
    }
    return canvas;
  }
  async function parse(file){
    if(file.size>220*1024*1024)throw new Error('PowerPoint is over the 220 MB import limit');
    const zip=await unzip(await file.arrayBuffer()),readText=async name=>{const b=await zip.read(name);return b?new TextDecoder().decode(b):null};
    const presentationText=await readText('ppt/presentation.xml');if(!presentationText)throw new Error('No presentation.xml was found');
    const presentation=new DOMParser().parseFromString(presentationText,'application/xml'),slideSize=first(presentation,'sldSz'),slideW=Number(slideSize?.getAttribute('cx')||12192000),slideH=Number(slideSize?.getAttribute('cy')||6858000);
    const presentationRels=await readText('ppt/_rels/presentation.xml.rels'),relMap=new Map();
    if(presentationRels){const relDoc=new DOMParser().parseFromString(presentationRels,'application/xml');desc(relDoc,'Relationship').forEach(rel=>relMap.set(rel.getAttribute('Id'),resolvePath('ppt/presentation.xml',rel.getAttribute('Target'))))}
    let slidePaths=desc(first(presentation,'sldIdLst'),'sldId').map(item=>relMap.get(item.getAttribute('r:id'))).filter(Boolean);
    if(!slidePaths.length)slidePaths=[...zip.entries.keys()].filter(name=>/^ppt\/slides\/slide\d+\.xml$/.test(name)).sort((a,b)=>Number(a.match(/slide(\d+)/)[1])-Number(b.match(/slide(\d+)/)[1]));
    if(!slidePaths.length)throw new Error('No readable slides were found');if(slidePaths.length>200)throw new Error('Presentation exceeds the 200-slide limit');
    const slides=[];
    for(let i=0;i<slidePaths.length;i++){
      const path=slidePaths[i],xml=await readText(path);if(!xml)continue;
      const doc=new DOMParser().parseFromString(xml,'application/xml'),bg=color(first(first(doc,'bg'),'solidFill'),'#ffffff'),elements=[];
      const relPath=path.split('/').slice(0,-1).join('/')+'/_rels/'+path.split('/').pop()+'.rels',rels=await readText(relPath),imageRels=new Map();
      if(rels){const relDoc=new DOMParser().parseFromString(rels,'application/xml');desc(relDoc,'Relationship').forEach(rel=>imageRels.set(rel.getAttribute('Id'),resolvePath(path,rel.getAttribute('Target'))))}
      const tree=first(doc,'spTree');
      for(const node of Array.from(tree?.children||[])){
        if(node.localName==='sp'){
          const box=boxFor(node,slideW,slideH);if(!box)continue;
          const props=first(node,'spPr'),fill=color(first(props,'solidFill'),null),paragraphs=desc(node,'p').map(par=>desc(par,'t').map(t=>t.textContent||'').join('')).filter(Boolean),run=first(node,'rPr')||first(node,'defRPr'),alignValue=first(node,'pPr')?.getAttribute('algn');
          elements.push({...box,kind:'shape',fill,text:paragraphs.join('\n'),fontSize:Number(run?.getAttribute('sz')||2400)/100,textColor:color(first(run,'solidFill'),'#171717'),bold:run?.getAttribute('b')==='1',italic:run?.getAttribute('i')==='1',align:alignValue==='ctr'?'center':alignValue==='r'?'right':'left'});
        }else if(node.localName==='pic'){
          const box=boxFor(node,slideW,slideH),blip=first(node,'blip'),rid=blip?.getAttribute('r:embed'),target=imageRels.get(rid);if(!box||!target)continue;
          const data=await zip.read(target);if(!data)continue;const ext=target.split('.').pop().toLowerCase(),mime=({jpg:'image/jpeg',jpeg:'image/jpeg',png:'image/png',gif:'image/gif',bmp:'image/bmp',svg:'image/svg+xml'})[ext]||'image/png',url=URL.createObjectURL(new Blob([data],{type:mime}));
          try{const image=new Image();await new Promise((resolve,reject)=>{image.onload=resolve;image.onerror=()=>reject(new Error('An embedded slide image could not be decoded'));image.src=url});elements.push({...box,kind:'image',image})}finally{URL.revokeObjectURL(url)}
        }
      }
      const title=elements.find(el=>el.text)?.text?.split('\n')[0]||`Slide ${i+1}`;slides.push({title,canvas:drawSlide(elements,bg)});
    }
    return slides;
  }
  window.LiteStreamPPTX={parse};
})();
