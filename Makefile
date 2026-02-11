scan:
	sudo losetup --partscan --find --show fat32.img

mkfs_fat:
	sudo mkfs.fat -F 32 -n TESTVOL $(LOOP)

do_mount:
	mkdir -p ./mnt_fat32
	sudo mount -o loop,offset=1048576 fat32.img ./mnt_fat32

check_losetup:
	losetup -l
