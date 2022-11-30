@echo off
cd /d "%~dp0"
javap -classpath "dejavu-engine\build\classes\kotlin\test" dejavu.engine.ConformanceTest
echo EXIT=%ERRORLEVEL%
java -cp "dejavu-engine\build\classes\kotlin\test;dejavu-engine\build\classes\kotlin\main;dejavu-language\build\libs\dejavu-language.jar;dejavu-types\build\libs\dejavu-types.jar;C:\Users\Administrator\.gradle\caches\modules-2\files-2.1\org.jetbrains.kotlin\kotlin-stdlib\2.0.21\618b539767b4899b4660a83006e052b63f1db551\kotlin-stdlib-2.0.21.jar" dejavu.engine.ConformanceTest
echo RUN_EXIT=%ERRORLEVEL%
gradlew.bat :dejavu-engine:test --tests dejavu.engine.ConformanceTest.t1CasesParseAndRenderFromExpectedIr
echo GRADLE_EXIT=%ERRORLEVEL%
