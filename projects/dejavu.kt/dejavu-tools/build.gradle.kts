plugins {
    kotlin("jvm")
    application
}

kotlin {
    jvmToolchain(21)
}

application {
    mainClass.set("dejavu.tools.MainKt")
}

dependencies {
    implementation(project(":dejavu"))
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
}
