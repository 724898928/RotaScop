cmake_minimum_required(VERSION 3.20)

set(CMAKE_VS_WINDOWS_TARGET_PLATFORM_VERSION "10.0.26100.0")

project(VirtualDisplayDriver LANGUAGES CXX)

set(CMAKE_MSVC_KERNEL_MODE TRUE)
set(CMAKE_MSVC_RUNTIME_LIBRARY "")
set(CMAKE_C_STANDARD_LIBRARIES "")
set(CMAKE_CXX_STANDARD_LIBRARIES "")

set(WDK_PATH "C:/PROGRA~2/WI3CF2~1/10" CACHE PATH "Path to WDK root (short 8.3 path)")
set(WDK_VERSION "10.0.26100.0" CACHE STRING "WDK version to use")
set(KMDF_VERSION "1.35" CACHE STRING "KMDF framework version")

set(WDK_INC_DIR "${WDK_PATH}/Include/${WDK_VERSION}")
set(WDK_LIB_DIR "${WDK_PATH}/Lib/${WDK_VERSION}/km/x64")

include_directories(
    ${WDK_INC_DIR}/km
    ${WDK_INC_DIR}/shared
    ${WDK_PATH}/Include/wdf/kmdf/${KMDF_VERSION}
    ${CMAKE_CURRENT_SOURCE_DIR}
)

add_executable(VirtualDisplayDriver
    Driver.cpp
    Adapter.cpp
    Device.cpp
    Monitor.cpp
    SwapChain.cpp
    Frame.cpp
    IddCxStubs.cpp
)

if(MSVC)
    set_target_properties(VirtualDisplayDriver PROPERTIES
        SUFFIX ".sys"
        LINK_FLAGS "/SUBSYSTEM:NATIVE /DRIVER /MANIFEST:NO /NODEFAULTLIB /ENTRY:DriverEntry"
    )
    target_compile_definitions(VirtualDisplayDriver PRIVATE
        _KERNEL_MODE=1
        _AMD64_
    )
    target_compile_options(VirtualDisplayDriver PRIVATE /kernel /Zc:threadSafeInit-)
endif()

target_link_libraries(VirtualDisplayDriver PRIVATE
    ${WDK_LIB_DIR}/ntoskrnl.lib
    ${WDK_LIB_DIR}/hal.lib
    ${WDK_LIB_DIR}/wmilib.lib
    ${WDK_LIB_DIR}/BufferOverflowK.lib
    ${WDK_PATH}/Lib/wdf/kmdf/x64/${KMDF_VERSION}/wdfdriverentry.lib
    ${WDK_PATH}/Lib/wdf/kmdf/x64/${KMDF_VERSION}/wdfldr.lib
)

option(BUILD_COMPANION "Build the user-mode companion service" ON)

if(BUILD_COMPANION)
    add_subdirectory(companion)
endif()
