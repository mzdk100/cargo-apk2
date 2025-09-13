package com.example.app;

import android.app.Activity;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.ServiceConnection;
import android.os.Bundle;
import android.os.IBinder;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

public class MainActivity extends Activity {
    static { System.loadLibrary("java_examples"); }
    // Declares the native method to be implemented in Rust.
    public native String getMessageFromRust();
    
    private MyService myService;
    private boolean isBound = false;
    
    private ServiceConnection connection = new ServiceConnection() {
        @Override
        public void onServiceConnected(ComponentName name, IBinder service) {
            MyService.LocalBinder binder = (MyService.LocalBinder) service;
            myService = binder.getService();
            isBound = true;
            Toast.makeText(MainActivity.this, "Service connected", Toast.LENGTH_SHORT).show();
        }

        @Override
        public void onServiceDisconnected(ComponentName name) {
            isBound = false;
            Toast.makeText(MainActivity.this, "Service disconnected", Toast.LENGTH_SHORT).show();
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // creates the main layout
        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);
        
        // adds the TextView containing the message from Rust
        TextView textView = new TextView(this);
        textView.setText(getMessageFromRust());
        textView.setLayoutParams(new LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT, 1f));
        layout.addView(textView);
        
        // adds service buttons
        addServiceButton(layout, "Start Service", v -> {
            startService(new Intent(this, MyService.class));
        });
        
        addServiceButton(layout, "Stop Service", v -> {
            stopService(new Intent(this, MyService.class));
        });
        
        addServiceButton(layout, "Bind Service", v -> {
            bindService(new Intent(this, MyService.class), 
                       connection, Context.BIND_AUTO_CREATE);
        });
        
        addServiceButton(layout, "Unbind Service", v -> {
            if (isBound) {
                unbindService(connection);
                isBound = false;
            }
        });
        
        addServiceButton(layout, "Call Service Method", v -> {
            if (isBound) {
                Toast.makeText(this, myService.getServiceInfo(), Toast.LENGTH_SHORT).show();
            } else {
                Toast.makeText(this, "Service not bound", Toast.LENGTH_SHORT).show();
            }
        });
        
        setContentView(layout);
    }
    
    private void addServiceButton(LinearLayout layout, String text, View.OnClickListener listener) {
        Button button = new Button(this);
        button.setText(text);
        button.setOnClickListener(listener);
        layout.addView(button);
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (isBound) {
            unbindService(connection);
            isBound = false;
        }
    }
}
