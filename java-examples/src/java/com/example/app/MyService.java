package com.example.app;

import android.app.Service;
import android.content.Intent;
import android.os.IBinder;
import android.util.Log;
import android.widget.Toast;

public class MyService extends Service {
    private static final String TAG = "MyService";

    // Binder given to clients
    private final IBinder binder = new LocalBinder();

    public class LocalBinder extends android.os.Binder {
        MyService getService() {
            return MyService.this;
        }
    }

    @Override
    public void onCreate() {
        super.onCreate();
        Log.d(TAG, "Service created");
        Toast.makeText(this, "Service created", Toast.LENGTH_SHORT).show();
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        Log.d(TAG, "Service started with startId: " + startId);
        Toast.makeText(this, "Service started", Toast.LENGTH_SHORT).show();

        // Simulate some background work
        new Thread(() -> {
            for (int i = 0; i < 5; i++) {
                try {
                    Thread.sleep(1000);
                    Log.d(TAG, "Doing work " + i);
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                }
            }
            stopSelf(); // Stop service when work is done
        }).start();

        return START_STICKY;
    }

    @Override
    public IBinder onBind(Intent intent) {
        Log.d(TAG, "Service bound");
        return binder;
    }

    @Override
    public boolean onUnbind(Intent intent) {
        Log.d(TAG, "Service unbound");
        return super.onUnbind(intent);
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        Log.d(TAG, "Service destroyed");
        Toast.makeText(this, "Service destroyed", Toast.LENGTH_SHORT).show();
    }

    // Example service method
    public String getServiceInfo() {
        return "MyService is running";
    }
}
