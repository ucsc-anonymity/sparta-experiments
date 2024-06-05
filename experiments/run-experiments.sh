#! /bin/bash

cd ..

python3 experiments/message_scaling.py
python3 experiments/submap_scaling.py
python3 experiments/user_and_message_scaling.py

cd -