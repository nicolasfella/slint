# Copyright © SixtyFPS GmbH <info@slint-ui.com>
# SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

cmake_minimum_required(VERSION 3.14)
project(qt_viewer LANGUAGES CXX)

if (NOT TARGET Slint::Slint)
    find_package(Slint REQUIRED)
endif()

find_package(Qt6 6.0 COMPONENTS Core Widgets QUIET)
if (NOT TARGET Qt::Widgets)
    find_package(Qt5 5.15 COMPONENTS Core Widgets)
endif()

if (TARGET Qt::Widgets)
    set(CMAKE_AUTOUIC ON)
    add_executable(qt_viewer qt_viewer.cpp)
    target_link_libraries(qt_viewer PRIVATE Slint::Slint Qt::Core Qt::Widgets)
endif()
