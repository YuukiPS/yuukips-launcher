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

Please implement this feature using appropriate state management and ensure it works seamlessly with the existing application architecture.