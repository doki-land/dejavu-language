@echo off
setlocal
cd /d "%~dp0"

echo === javap ConformanceTest ===
javap -classpath "dejavu-engine\build\classes\kotlin\test" dejavu.engine.ConformanceTest
if errorlevel 1 exit /b 1

echo === gradle test with JUnit4 attempt will be separate ===
call gradlew.bat :dejavu-engine:test --no-daemon > gradle-test.log 2>&1
echo GRADLE_EXIT=%ERRORLEVEL%
type gradle-test.log | findstr /i "PASSED FAILED SUCCESSFUL ClassNotFound tests completed BUILD"
exit /b %ERRORLEVEL%
