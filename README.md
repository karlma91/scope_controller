retrolink n64 usb controller uuid:
03000000-7900-0000-0600-000010010000
Controller sdl db line
03000000790000000600000010010000,RetroLink N64,x:b3,a:b2,b:b1,y:b0,back:b8,start:b9,dpleft:h0.8,dpdown:h0.4,dpright:h0.2,dpup:h0.1,leftshoulder:b4,lefttrigger:b6,rightshoulder:b5,righttrigger:b7,leftstick:b10,rightstick:b11,leftx:a0,lefty:a3,rightx:a1,righty:a4,platform:Linux,

to update the release
cargo build --release
systemctl start telescope.service