cmake_minimum_required(VERSION 3.20)

project(VirtualDisplayDriver LANGUAGES CXX)

set(CMAKE_MSVC_KERNEL_MODE TRUE)

set(WDK_PATH "C:/Program Files (x86)/Windows Kits/10" CACHE PATH "Path to WDK root")
set(WDK_VERSION "10.0.22621.0" CACHE STRING "WDK version to use")

set(WDK_INC_DIR "${WDK_PATH}/Include/${WDK_VERSION}")
set(WDK_LIB_DIR "${WDK_PATH}/Lib/${WDK_VERSION}/km/x64")

include_directories(
    ${WDK_INC_DIR}/km
    ${WDK_INC_DIR}/shared
    ${WDK_INC_DIR}/um
    ${CMAKE_CURRENT_SOURCE_DIR}
)

add_executable(VirtualDisplayDriver
    Driver.cpp
    Adapter.cpp
    Device.cpp
    Monitor.cpp
    SwapChain.cpp
    Frame.cpp
)

if(MSVC)
    set_target_properties(VirtualDisplayDriver PROPERTIES
        SUFFIX ".sys"
        LINK_FLAGS "/SUBSYSTEM:NATIVE /DRIVER"
    )
endif()

target_link_libraries(VirtualDisplayDriver PRIVATE
    ${WDK_LIB_DIR}/ntoskrnl.lib
    ${WDK_LIB_DIR}/hal.lib
    ${WDK_LIB_DIR}/dxgkrnl.lib
    ${WDK_LIB_DIR}/wmilib.lib
)

option(BUILD_COMPANION "Build the user-mode companion service" ON)

if(BUILD_COMPANION)
    add_subdirectory(companion)
endif()
