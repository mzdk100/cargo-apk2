package com.example.app

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView

class MainActivity : Activity() {
    private lateinit var textView: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Create a LinearLayout as the container
        val layout = LinearLayout(this)
        layout.orientation = LinearLayout.VERTICAL

        // Create a TextView programmatically
        textView = TextView(this)
        textView.text = "Hello from Kotlin!"
        textView.textSize = 18f
        textView.setPadding(20, 30, 20, 30)
        textView.layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)

        // Create a button to start the service
        val button = Button(this)
        button.text = "Start Service"
        button.setOnClickListener {
            val intent = Intent(this, MyService::class.java)
            startService(intent)
            textView.text = "Service started!"
        }

        // Add views to the layout
        layout.addView(textView)
        layout.addView(button)

        // Set the layout as the content view
        setContentView(layout)
    }
}
