# Replace game data list with API
So every time the launcher is run, it will always load GET https://ps.yuuki.me/json/game_all.json?time={randomtime} If it fails, it just displays, `Sorry, your internet is having problems or our server is having problems, please try again later.`

Respon:
```json
[
    {
        "id": 1,
        "slug": "genshin-impact",
        "title": "Genshin Impact",
        "description": "Experience Genshin Impact like never before with our high-quality private server. Unlock all characters, weapons, and explore Teyvat with custom features and enhanced gameplay.",
        "keyword": "Genshin Impact, Private Server, Emulator, Teyvat, Quests, Characters, Weapons, Artifacts",
        "lastUpdate": 1751117160,
        "image": "https://book-api.yuuki.me/image/blog/110000000_1751024349123_small.webp",
        "thumbnail": "https://book-api.yuuki.me/image/blog/110000000_1751024349123_thumbnail.webp",
        "icon": "https://book-api.yuuki.me/image/blog/110000000_1751115608065_small.webp",
        "engine": [
            {
                "id": 1,
                "name": "Grasscutter",
                "short": "GC",
                "description": "A high-performance Genshin Impact private server emulator.",
                "version": "?.?.?",
                "versionSupport": {
                    "4.0.0": [
                        1,
                        2,
                        3
                    ],
                    "4.0.1": [
                        1,
                        2,
                        3
                    ],
                    "5.6.0": [
                        1,
                        2
                    ],
                    "5.7.0": [
                        1
                    ]
                },
                "link": "https://github.com/Grasscutters/Grasscutter",
                "command": 2,
                "features": [
                    "Quests here can only be started manually and are buggy but you can get updates.",
                    "Server not stable and may crash",
                    "Commands available via in-game chat and web interface",
                    "No Events",
                    "Abyss working but outdated",
                    "Domain sometimes working but buggy",
                    "Serenitea pot working but buggy",
                    "Character heal or ability maybe not work",
                    "food buff and other effects may not work",
                    "Character stats not accurate?",
                    "Artifacts cutomization available via web interface"
                ]
            },
            {
                "id": 2,
                "name": "Genshin Impact Official",
                "short": "GIO",
                "description": "Leaked official version of private server.",
                "version": "3.2.0",
                "versionSupport": {
                    "3.2.0": [
                        1,
                        2
                    ]
                },
                "link": "https://discord.gg/u7fGFjnF",
                "command": 1,
                "features": [
                    "Quests perfectly working only up to 3.2 version",
                    "Stabil server",
                    "Commands only available via web interface and limited",
                    "Custom events available",
                    "No Abyss",
                    "Domains working and you can enter all day",
                    "Serenitea pot working",
                    "Character heal and ability working",
                    "Food buff and other effects working",
                    "Accurate character stats",
                    "Artifacts customization available via web interface"
                ]
            }
        ]
    },
    {
        "id": 2,
        "slug": "star-rail",
        "title": "Honkai Star Rail",
        "description": "Journey through the stars with our Honkai Star Rail private server. Experience the complete story with all characters, light cones, and unlimited stellar jade.",
        "keyword": "Honkai Star Rail, Private Server, Emulator, Stellar Jade, Characters, Light Cones, Quests",
        "lastUpdate": 1749079380,
        "image": "https://book-api.yuuki.me/image/blog/110000000_1751023300485_small.webp",
        "thumbnail": "https://book-api.yuuki.me/image/blog/110000000_1751023300485_thumbnail.webp",
        "icon": "https://book-api.yuuki.me/image/blog/110000000_1751118500664_small.webp",
        "engine": [
            {
                "id": 5,
                "name": "LunarCore",
                "short": "LC",
                "description": "A powerful and flexible Honkai Star Rail private server emulator.",
                "version": "?.?.?",
                "versionSupport": {
                    "3.3.3": [
                        1,
                        2,
                        3
                    ]
                },
                "link": "https://github.com/Melledy/LunarCore",
                "command": 2
            }
        ]
    },
    {
        "id": 3,
        "slug": "blue-archive",
        "title": "Blue Archive",
        "description": "Command your students in Blue Archive private server with unlimited resources, all characters unlocked, and exclusive content not available in the official version.",
        "keyword": "Blue Archive, Private Server, Emulator, Students, Characters, Resources, Quests",
        "lastUpdate": 1749079380,
        "image": "https://book-api.yuuki.me/image/blog/110000000_1751024665133_small.webp",
        "thumbnail": "https://book-api.yuuki.me/image/blog/110000000_1751024665133_thumbnail.webp",
        "icon": "https://book-api.yuuki.me/image/blog/110000000_1751115608065_small.webp",
        "engine": [
            {
                "id": 6,
                "name": "BaPs",
                "short": "BA",
                "description": "A robust Blue Archive private server emulator with extensive features.",
                "version": "?.?.?",
                "versionSupport": {
                    "1.57": [
                        2,
                        3
                    ]
                },
                "link": "https://github.com/gucooing/BaPs",
                "command": 1
            }
        ]
    }
]
```
what you need to know:
- id = id game, check `TypeGame`
- image = image background
- thumbnail = image thumbnail
- icon = image icon
- description = info about game
- lastUpdate = last update game
every game has different engine and versionSupport:PlatformType so basically:
`game -> engine -> versionSupport:PlatformType`
So every time you click the `Start Game` button it will display a menu to select the available engine and available version and Only display PlatformType 1 data because this is only for pc
