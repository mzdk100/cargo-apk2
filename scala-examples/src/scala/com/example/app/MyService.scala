package com.example.app

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log

class MyService extends Service {
  private val TAG = "MyService"

  override def onCreate(): Unit = {
    super.onCreate()
    Log.d(TAG, "Service created")
  }

  override def onStartCommand(intent: Intent, flags: Int, startId: Int): Int = {
    Log.d(TAG, "Service started")

    // Do some work here
    new Thread(new Runnable {
      override def run(): Unit = {
        // Simulate a long-running task
        try {
          Thread.sleep(5000)
          Log.d(TAG, "Task completed")
        } catch {
          case e: InterruptedException => e.printStackTrace()
        }
      }
    }).start()

    Service.START_STICKY
  }

  override def onBind(intent: Intent): IBinder = null

  override def onDestroy(): Unit = {
    super.onDestroy()
    Log.d(TAG, "Service destroyed")
  }
}
