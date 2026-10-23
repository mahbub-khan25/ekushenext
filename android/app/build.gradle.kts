import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

// The release name comes from the desktop release file (.github/release/VERSION), e.g. v0.1.0-beta.11
val release = rootProject.file("../.github/release/VERSION").readText().trim().removePrefix("v")
// A growing whole number for Android: 0.1.0-beta.11 -> 100011 (betas count up inside a version)
val versionNumber = Regex("""(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+))?""").find(release)!!.destructured.let { (a, b, c, beta) ->
    a.toInt() * 10_000_000 + b.toInt() * 100_000 + c.toInt() * 1_000 + (beta.toIntOrNull() ?: 999)
}

// Signing for release builds: android/keystore.properties (not in git) or environment variables in CI
val signing = Properties().apply {
    val f = rootProject.file("keystore.properties")
    if (f.exists()) f.inputStream().use { load(it) }
}
fun secret(key: String, env: String): String? = signing.getProperty(key) ?: System.getenv(env)

android {
    namespace = "com.ekushenext.keyboard"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.ekushenext.keyboard"
        minSdk = 24
        targetSdk = 35
        versionCode = versionNumber
        versionName = release
    }

    signingConfigs {
        create("release") {
            val store = secret("storeFile", "EKUSHE_KEYSTORE")
            if (store != null) {
                storeFile = file(store)
                storePassword = secret("storePassword", "EKUSHE_KEYSTORE_PASSWORD")
                keyAlias = secret("keyAlias", "EKUSHE_KEY_ALIAS")
                keyPassword = secret("keyPassword", "EKUSHE_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            if (secret("storeFile", "EKUSHE_KEYSTORE") != null) signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }

    androidResources {
        // The dictionary is already compressed
        noCompress += "z"
    }
    lint {
        checkReleaseBuilds = false
    }
    packaging {
        jniLibs.useLegacyPackaging = false
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
}
