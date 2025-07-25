package com.example.app

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import scala.compiletime.uninitialized

class MainActivity extends Activity {
  private var textView: TextView = uninitialized

  override def onCreate(savedInstanceState: Bundle): Unit = {
    super.onCreate(savedInstanceState)

    // Create a LinearLayout as the container
    val layout = new LinearLayout(this)
    layout.setOrientation(LinearLayout.VERTICAL)

    // Create a TextView programmatically
    textView = new TextView(this)
    textView.setText("Hello from Scala!")
    textView.setTextSize(18f)
    textView.setPadding(20, 30, 20, 30)
    textView.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))

    // Create a button to start the service
    val button = new Button(this)
    button.setText("Start Service")
    button.setOnClickListener(_ => {
      val intent = new Intent(this, classOf[MyService])
      startService(intent)
      textView.setText("Service started!")
    })

    // Add views to the layout
    layout.addView(textView)
    layout.addView(button)

    // Set the layout as the content view
    setContentView(layout)
  }
}
