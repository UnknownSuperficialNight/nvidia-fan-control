#!/bin/bash
#USER VARIABLES INPUT HERE
fan_amount=1 # this is my fan ammount change it to your gpu fan amount
refresh_time=5 # this is the amount of time to refresh (in seconds)
speed=("59" "70" "80" "90" "93" "95" "97" "100")
#USER VARIABLES END
check_privileges() {
    if (( EUID != 0 )); then
        echo "User is not running with root privileges"
        exit
    fi
}
check_privileges

cols=$( tput cols )
rows=$( tput lines )
 middle_row=$(( rows / 2 ))
 middle_col=$(( (cols /2) - 4 ))

temp_func () {
temp=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader)
}

temp_loop () {
diff=999
for x in "${speed[@]}"; do
cd=$((temp - x))
nd=${cd#-}
if test "$nd" -lt "$diff"; then
   speed_output=$x
   diff=$nd
fi; done
if [[ "$temp" -ge '80' ]];then
speed_output=$((speed_output + 20))
fi
if [[ "$speed_output" -gt '100' ]]; then speed_output='100' ;fi
}

#main_loop
for (( ; ; ));do
sleep "$refresh_time"
clear
temp_func
tput cup $middle_row $middle_col
echo "$temp" temp
temp_loop
tput cup $((middle_row +8)) $middle_col
if [[ "$speed_output" == "$temp_capture" ]]; then
echo skipped
else
for faninc in $(eval echo "{1..$fan_amount}") ;do fan_increment=$((faninc - 1)) && nvidia-settings -a GPUTargetFanSpeed[fan:$fan_increment]="$speed_output" ;done
fi
tput cup $((middle_row +1)) $middle_col
echo "$speed_output" speed_output
tput cup $((middle_row +2)) $middle_col
echo "$temp_capture" temp_capture
temp_capture=$speed_output
done

