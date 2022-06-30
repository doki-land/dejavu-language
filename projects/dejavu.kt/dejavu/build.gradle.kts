plugins {
    kotlin("jvm")
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    api(project(":dejavu-engine"))
    api(project(":dejavu-language"))
    api(project(":dejavu-types"))
    api("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
}
