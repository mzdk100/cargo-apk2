package com.example.app;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public class MainActivity extends Activity {
    // 加载包含本地方法的库
    static {
        System.loadLibrary("example_app");
    }

    // 声明本地方法，将在Rust中实现
    public native String getMessageFromRust();

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // 创建一个TextView来显示消息
        TextView textView = new TextView(this);
        textView.setText(getMessageFromRust());

        // 设置TextView为主视图
        setContentView(textView);
    }
}
