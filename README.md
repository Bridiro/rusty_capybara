# Rusty capybara

## Introduction

The project focuses on building a program that guide a robot (partecipating in the Robocup Junior Rescue Maze). We want to build it in Rust, a blazingly fast language that, with the help of some crates, will help us to interface all the needed sensors and the cameras for the computer vision.

## Index

-   [Sensors](#sensors)
-   [Mapping](#mapping)
-   [Vision](#vision)
-   [PIDs](#pids)

### <a id="sensors"></a>Sensors

We're using **VL8160X time-of-flight** sensors, that provides the perfect accuracy and range for the project. They uses **i2c** as communication protocol, so we can have 6 on the same bus. On **i2c** we have the **IMU** as well, it is the **MPU6050**. With that and with the sensors we can use **algorithms** that will help our robot explore the maze in a safe way.

### <a id="mapping"></a>Mapping

The goal is to have the whole labirinth explored, and to archieve this, we need to map it. The algorith is still to decide, but we want it to be flexible and to work in every condition. The maze can contains **checkpoints**, **black tiles**, **blue tiles** and **victims**. This is the **RESCUE MAZE** so the very goal here is to find all the victims. In the map we will also store where victims are, so we can skip them if we encounter the same 2 times.

### <a id="vision"></a>Vision

To find the victims we need **computer vision**. Computer vision is a brach of **AI** that focuses on finding **matching patterns** in arrays (in this case **images**). This images are retrived by opencv, and using a **YOLOv7** model by **Ultralytics** we can try to find the victims. These victims can be **a green square**, **a red square**, **a yellow square**, **an H**, **an S** or **an U**. They are attached to the walls, so we need cameras on both sides of the robot to see them. Loading the model in Rust is accomplished thanks to the crate [od_opencv](https://crates.io/crates/od_opencv).

### <a id="pids"></a>PIDs

Based on our previous experiences in the competition, using PIDs to calculate trajectories is the best thing to do, because the movements will always be near perfect so the robot can move without hitting any wall.
