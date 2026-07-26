cmake_minimum_required(VERSION 3.20)
project(VirtualDisplayDriver)

set(CMAKE_SYSTEM_NAME Windows)
set(CMAKE_SYSTEM_VERSION 10.0)

# WDK paths - adjust as needed for your environment
if(NOT DEFINED WDK_PATH)
    if(DEFINED ENV{WDK_PATH})
        set(WDK_PATH $ENV{WDK_PATH})
    elseif(EXISTS "C:/Program Files (x86)/Windows Kits/10")
        set(WDK_PATH "C:/Program Files (x86)/Windows Kits/10")
    else()
        message(FATAL_ERROR "WDK_PATH not found. Please set WDK_PATH or install Windows WDK.")
    endif()
endif()

# Find the latest WDK version
file(GLOB WDK_VERSIONS RELATIVE ${WDK_PATH}/Include
    "${WDK_PATH}/Include/10.0.*")
list(SORT WDK_VERSIONS)
list(REVERSE WDK_VERSIONS)
if(WDK_VERSIONS)
    list(GET WDK_VERSIONS 0 WDK_VERSION)
else()
    message(FATAL_ERROR "No WDK version found in ${WDK_PATH}/Include/10.0.*")
endif()

message(STATUS "Using WDK version: ${WDK_VERSION}")
message(STATUS "WDK Path: ${WDK_PATH}")

# WDK include directories
set(WDK_INCLUDES
    ${WDK_PATH}/Include/${WDK_VERSION}/km
    ${WDK_PATH}/Include/${WDK_VERSION}/km/crt
    ${WDK_PATH}/Include/${WDK_VERSION}/shared
    ${WDK_PATH}/Include/${WDK_VERSION}/um
)

# WDK library directories
set(WDK_LIB_DIR
    ${WDK_PATH}/Lib/${WDK_VERSION}/km/x64
)

add_library(VirtualDisplayDriver SHARED
    Driver.cpp
    Adapter.cpp
    Device.cpp
    Monitor.cpp
    SwapChain.cpp
    Frame.cpp
)

target_include_directories(VirtualDisplayDriver PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${WDK_INCLUDES}
)

target_compile_definitions(VirtualDisplayDriver PRIVATE
    _WIN32_WINNT=0x0A00
    _X86_
    _AMD64_
    DBG=1
)

target_compile_options(VirtualDisplayDriver PRIVATE
    /W4
    /WX
    /kernel
    /GS
)

target_link_directories(VirtualDisplayDriver PRIVATE ${WDK_LIB_DIR})
target_link_libraries(VirtualDisplayDriver PRIVATE
    ntoskrnl.lib
    hal.lib
    wdmsec.lib
    ndis.lib
)

# Output as .sys file
set_target_properties(VirtualDisplayDriver PROPERTIES
    SUFFIX ".sys"
    PREFIX ""
    COMPILE_FLAGS "/EHs-c- /GR-"
)
