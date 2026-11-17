package com.oxide.engine;

import com.google.androidgamesdk.GameActivity;

/**
 * Android entry point.
 *
 * GameActivity loads the shared library named by the
 * {@code android.app.lib_name} manifest metadata and calls its exported
 * {@code android_main} symbol. Bevy's {@code #[bevy_main]} macro generates that
 * symbol for the cdylib built by {@code cargoBuild}.
 */
public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("fnf_oxide_engine");
    }
}
