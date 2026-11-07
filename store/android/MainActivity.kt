package __PACKAGE__

import android.content.ContentValues
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.MediaStore
import android.util.Base64
import android.view.View
import android.view.WindowManager
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStream

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    // Keep the screen on while the studio is open (recording and presenting).
    window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    // Keep the UI clear of the status bar, navigation bar and camera cut-out.
    val root = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      insets
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    webView.settings.mediaPlaybackRequiresUserGesture = false
    webView.addJavascriptInterface(LiteBridge(this), "LiteAndroid")
  }
}

/** Small native helpers the web UI calls: save a recording to Downloads and open links. */
class LiteBridge(private val activity: MainActivity) {
  private var stream: OutputStream? = null
  private var pendingUri: Uri? = null
  private var pendingFile: File? = null
  private var label: String = ""

  @JavascriptInterface
  fun beginSave(name: String, mime: String): Boolean {
    abortSave()
    val safe = name.replace(Regex("[^A-Za-z0-9._-]"), "_")
    return try {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        val values = ContentValues().apply {
          put(MediaStore.MediaColumns.DISPLAY_NAME, safe)
          put(MediaStore.MediaColumns.MIME_TYPE, if (mime.isNotEmpty()) mime.substringBefore(';') else "video/webm")
          put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS + "/LiteStream")
          put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val uri = activity.contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values)
          ?: return false
        pendingUri = uri
        stream = activity.contentResolver.openOutputStream(uri)
        label = "Downloads/LiteStream/$safe"
      } else {
        val dir = activity.getExternalFilesDir(Environment.DIRECTORY_MOVIES) ?: activity.filesDir
        val file = File(dir, safe)
        pendingFile = file
        stream = FileOutputStream(file)
        label = "app storage (Movies)/$safe"
      }
      stream != null
    } catch (e: Exception) {
      abortSave()
      false
    }
  }

  @JavascriptInterface
  fun appendChunk(base64: String): Boolean {
    return try {
      stream?.write(Base64.decode(base64, Base64.DEFAULT))
      stream != null
    } catch (e: Exception) {
      false
    }
  }

  @JavascriptInterface
  fun endSave(): String {
    try {
      stream?.flush()
      stream?.close()
    } catch (e: Exception) {
    }
    stream = null
    val uri = pendingUri
    if (uri != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
      val done = ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) }
      activity.contentResolver.update(uri, done, null, null)
    }
    pendingUri = null
    pendingFile = null
    return label
  }

  @JavascriptInterface
  fun abortSave() {
    try {
      stream?.close()
    } catch (e: Exception) {
    }
    stream = null
    pendingUri?.let { activity.contentResolver.delete(it, null, null) }
    pendingFile?.delete()
    pendingUri = null
    pendingFile = null
  }

  @JavascriptInterface
  fun openUrl(url: String) {
    if (!(url.startsWith("https://") || url.startsWith("http://") || url.startsWith("mailto:"))) return
    activity.runOnUiThread {
      try {
        activity.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
      } catch (e: Exception) {
      }
    }
  }
}
