# Downloader Manager
Create a Download Manager page with the following requirements:

1. User Interface:
- Add a "Downloads" link in the navigation bar (check Header.tsx) next to the debug icon
- Design a responsive table/list view to display downloading files
- Include a clean, modern interface with clear visual hierarchy

2. Download Item Display:
For each downloading item, show:
- File name with extension
- Total file size (in appropriate units: KB, MB, GB)
- Progress bar showing download completion percentage
- Current download speed (KB/s, MB/s)
- Status indicator (Downloading, Paused, Completed, Error)
- Time remaining for download completion
- File type icon based on extension

3. Control Features:
For each download:
- Pause/Resume button (toggle functionality)
- Stop/Cancel button
- Restart button for failed downloads
- Option to clear completed downloads
- Bulk actions for multiple selected downloads

4. Additional Functionality:
- Real-time progress updates
- Persistent download history across sessions
- Search/filter downloads by name, status, or date
- Sort downloads by different parameters (name, size, progress, etc.)
- Show total downloads in progress and completed
- Option to open download location
- Support for resuming interrupted downloads

5. Error Handling:
- Display meaningful error messages
- Provide retry options for failed downloads
- Show warning before canceling active downloads

6. API Functions:
- Pause/Resume Download
- Stop/Cancel Download
- Restart Download
- Clear Completed Downloads
- Get Download Status
- Get Download History
- Search Download History

# Download Game Data
So create a `Download Game` button, if you click it it will do an API check on:
https://ps.yuuki.me/game/download/pc/[id_game]/[id_channel]/[version].json
when it is called it will display the response, 
when ok:
```json
{"message":"","metode":1,"file":[{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.zip","file":"GI-3.2-PC-YuukiPS.zip","md5":"3676b6a605aed092adbf9c21fe852322","package_size":2193570942},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z01","file":"GI-3.2-PC-YuukiPS.z01","md5":"05c1972ac169b08362138c30b40e69a4","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z02","file":"GI-3.2-PC-YuukiPS.z02","md5":"efc5c701e21ab93c5944bc48d068f1e4","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z03","file":"GI-3.2-PC-YuukiPS.z03","md5":"cc49624be840093989eb6b85563e592c","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z04","file":"GI-3.2-PC-YuukiPS.z04","md5":"e662225f82d950c76404d2b6afc21062","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z05","file":"GI-3.2-PC-YuukiPS.z05","md5":"c37172268a758ba91a11644e16b09842","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z06","file":"GI-3.2-PC-YuukiPS.z06","md5":"d4c74f9aab3d158504fcccce73770050","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z07","file":"GI-3.2-PC-YuukiPS.z07","md5":"b07bddd7336c7468b7ac1f010b3656db","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z08","file":"GI-3.2-PC-YuukiPS.z08","md5":"c25a1e48bf3626d98dfb7210ccc2b88a","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z09","file":"GI-3.2-PC-YuukiPS.z09","md5":"f52f92529dd005324e73d63e1122fcea","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z10","file":"GI-3.2-PC-YuukiPS.z10","md5":"c79c5116d06d8d24b8e3c045e15528a2","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z11","file":"GI-3.2-PC-YuukiPS.z11","md5":"c0aca02a897a0b00bbe927325059a1b7","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z12","file":"GI-3.2-PC-YuukiPS.z12","md5":"edb7e8202e1208d398600dd96e4943d6","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z13","file":"GI-3.2-PC-YuukiPS.z13","md5":"1be7b1268aecc0fc0db2dbd5af2e65d8","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z14","file":"GI-3.2-PC-YuukiPS.z14","md5":"3b9fb1884e394f8c9860c5be95e49790","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z15","file":"GI-3.2-PC-YuukiPS.z15","md5":"3f6a0aea55009e4b16b4791d6905e85a","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z16","file":"GI-3.2-PC-YuukiPS.z16","md5":"4526d5858f90812176609c6d522e5df4","package_size":3221225472},{"url":"https://file.yuuki.me/p/Local/Project/GenshinImpact/Data/PC/3.2.0/Global/OneClick/GI-3.2-PC-YuukiPS.z17","file":"GI-3.2-PC-YuukiPS.z17","md5":"406139a801624e643e60cf230730bbd2","package_size":3221225472}]}
```
whene bad:
``` json
{"retcode":-1,"message":"Not found"}
```
If successful it will be divided into several methods, check below:
## Metode 1
1. which displays a popup list of files that must be downloaded including info on file name, size, md5 and the total of all files.
2. display select the folder where you want to place the zip file, and don't forget to calculate the estimated size of all file parts in GB including when unzipping (zip + unzip file)
3. (fix) Failed to select folder: Command select_folder not found, When the user successfully selects a folder, it will check the files in that folder. If the same file is found, try matching the MD5. If it matches, give a correct mark.

Please implement this feature using appropriate state management and ensure it works seamlessly with the existing application architecture.