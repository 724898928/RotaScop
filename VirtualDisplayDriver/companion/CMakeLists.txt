cmake_minimum_required(VERSION 3.20)

project(RotaScopeCompanion LANGUAGES CXX)

add_executable(RotaScopeCompanion
    main.cpp
)

target_link_libraries(RotaScopeCompanion PRIVATE
    ws2_32
)
