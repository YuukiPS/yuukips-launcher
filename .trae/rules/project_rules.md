* don't check build code, just complete the code without running `npm run tauri dev` or `npm run dev` for live (so skip live)
* if build / Building is in progress always check back after 1 minute, if it takes too long (after trying 1x, just skip the testing)
* If the code can be used as a component, create a new file and then combine it in a folder group.
* If the file is more than 1000 line, try refactoring some functions that can be made into components so that the file is smaller.