#!/usr/bin/env zsh
rm -rf test.mlib
current_dir=$(pwd)
api_url="http://localhost:22110/api"
uuid=$(curl -s -X GET "$api_url/create_library?path=$current_dir&library_name=test")
echo Result: "$uuid"
uuid=$(echo "$uuid" | jq .library_uuid | sed 's/\"//g')
echo Library Uuid: "$uuid"
result=$(curl -s -X GET "$api_url/add_media?uuid=$uuid&path=$current_dir/test/1.jpg&kind=Image")
echo Result: "$result"