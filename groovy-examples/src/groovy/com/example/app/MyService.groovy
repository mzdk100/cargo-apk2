package com.example.app

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log

class MyService extends Service {
    private static final String TAG = "MyService"

    @Override
    void onCreate() {
        super.onCreate()
        Log.d(TAG, "Service created")
    }

    @Override
    int onStartCommand(Intent intent, int flags, int startId) {
        Log.d(TAG, "Service started")

        // Do some work here
        Thread.start {
            // Simulate a long-running task
            try {
                Thread.sleep(5000)
                Log.d(TAG, "Task completed")
            } catch (InterruptedException e) {
                e.printStackTrace()
            }
        }

        return START_STICKY
    }

    @Override
    IBinder onBind(Intent intent) {
        return null
    }

    @Override
    void onDestroy() {
        super.onDestroy()
        Log.d(TAG, "Service destroyed")
    }
}
