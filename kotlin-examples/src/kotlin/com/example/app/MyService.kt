package com.example.app

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log

class MyService : Service() {
    private val TAG = "MyService"

    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Service created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "Service started")

        // Do some work here
        Thread {
            // Simulate a long-running task
            try {
                Thread.sleep(5000)
                Log.d(TAG, "Task completed")
            } catch (e: InterruptedException) {
                e.printStackTrace()
            }
        }.start()

        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d(TAG, "Service destroyed")
    }
}
