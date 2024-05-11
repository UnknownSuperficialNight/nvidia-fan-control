#!/bin/bash

# Note: This Bash script is a first draft completed in approximately 30 minutes, intended to convey the basic concept of the idea.
# It is fully functional but hasn't been updated or optimized. It's provided primarily for those who prefer Bash scripts over the Rust binary
# or for individuals who want to grasp the concept of the Rust version without understanding Rust code.

#USER VARIABLES INPUT HERE
fan_amount=1                                     # This is my fan amount change it to your gpu fan amount
refresh_time=5                                   # This is the amount of time to refresh (in seconds)
speed=("10" "20" "30" "40" "59" "70" "80" "100") # This is the array used to set target fan speeds
#USER VARIABLES END
check_privileges() {
    if ((EUID != 0)); then
        echo "User is not running with root privileges"
        exit
    fi
}
check_privileges

cols=$(tput cols)
rows=$(tput lines)
middle_row=$((rows / 2))
middle_col=$(((cols / 2) - 4))

temp_func() {
    temp=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader)
}

temp_loop() {
    diff=999
    for x in "${speed[@]}"; do
        cd=$((temp - x))
        nd=${cd#-}
        if test "$nd" -lt "$diff"; then
            speed_output=$x
            diff=$nd
        fi
    done
    if [[ "$temp" -ge '80' ]]; then
        speed_output=$((speed_output + 20))
    fi
    if [[ "$speed_output" -gt '100' ]]; then speed_output='100'; fi
}

#main_loop
for (( ; ; )); do
    sleep "$refresh_time"
    clear
    temp_func
    tput cup $middle_row $middle_col
    echo "$temp" temp
    temp_loop
    tput cup $((middle_row + 8)) $middle_col
    if [[ "$speed_output" == "$temp_capture" ]]; then
        echo skipped
    else
        for faninc in $(eval echo "{1..$fan_amount}"); do fan_increment=$((faninc - 1)) && nvidia-settings -a GPUTargetFanSpeed[fan:$fan_increment]="$speed_output"; done
    fi
    tput cup $((middle_row + 1)) $middle_col
    echo "$speed_output" speed_output
    tput cup $((middle_row + 2)) $middle_col
    echo "$temp_capture" temp_capture
    temp_capture=$speed_output
done
