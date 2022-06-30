plugins {
    kotlin("jvm")
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation(project(":dejavu-types"))
    implementation(project(":dejavu-language"))
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")
    // JUnit4 runner avoids Gradle 9.4 + non-ASCII path ClassNotFoundException
    // with the JUnit Platform worker ClassLoader.
    testImplementation(project(":dejavu"))
    testImplementation(kotlin("test-junit"))
}

tasks.test {
    useJUnit()
}
