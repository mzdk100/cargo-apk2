package com.example.app

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView

class MainActivity extends Activity {
    private TextView textView

    @Override
    void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState)

        // Create a LinearLayout as the container
        def layout = new LinearLayout(this)
        layout.orientation = LinearLayout.VERTICAL

        // Create a TextView programmatically
        textView = new TextView(this)
        textView.text = "Hello from Groovy!"
        textView.textSize = 18f
        textView.setPadding(20, 30, 20, 30)

        // Create a button to start the service
        def button = new Button(this)
        button.text = "Start Service"
        button.onClickListener = {
            def intent = new Intent(this, MyService.class)
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
