plugins {
    kotlin("jvm")
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation(project(":dejavu-types"))
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
}
