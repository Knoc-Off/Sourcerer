# Sourcerer


- ## Issue it solves
	- I was frustrated with having to move all of my configurations every time I distro hopped or reinstalled. With this you only have to copy one folder and execute the program.
		- Very worthwhile for me. Probably has limited use for others.


- ## What it does
	- It copies files/folders to a local folder and places a symlink where it was. Then you can move the folder to a new computer and have all of your programs configured in seconds.


- ## How to use
	- type ./sourcerer when inside the directory that you want to be the local file. running it without any perimeters just re-establishes symlinks.
	- -a - adds a new folder to the local.
	- -r - Removes an alias. it 'soft deletes it' so your config replaces the symlink and was as if it was never added to sorcerer
	- -l - A *very* rudimentary list. This needs to be vastly improved. A search feature is planned

- ## Planned Features
	- Better list functionality as well as a search function
	- Hntegrate sudo/ elevated permissions 
	- Have a remote executable, as well as a configurable source folder
	- Improved error handling

- ## Issues
	- Still a bit iffy with sudo, be wary


- ## Compatibility
	- Linux only, windows is bad

