# Skelly-Jelly Quick Start Guide

## 🚀 Getting Started in 5 Minutes

### Prerequisites
- Python 3.13 or higher
- Git
- macOS 14+ (primary platform) or Windows 10/11 or Linux

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd skelly-jelly

# Install uv if you don't have it
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install dependencies
uv sync

# Run the hello world
uv run python main.py
```

## 🎯 What is Skelly-Jelly?

Skelly-Jelly is your ADHD-friendly focus companion - a cute melty skeleton that:
- 🧠 Detects when you're distracted or hyperfocused
- 🎮 Provides gentle nudges without disrupting flow
- 💡 Offers context-aware help for coding, writing, and design
- 🔒 Runs entirely on your device for privacy

## 🏃‍♀️ First Run

When you first run Skelly-Jelly:

1. **Companion appears** - A small melty skeleton in the corner of your screen
2. **Permission prompts** - Grant accessibility permissions for monitoring
3. **Initial calibration** - Works normally for 30 minutes to learn your patterns
4. **First intervention** - Gentle animation when distraction detected

## ⚙️ Basic Configuration

Create a `config.json` file:

```json
{
  "intervention_settings": {
    "min_interval_ms": 900000,  // 15 minutes between nudges
    "work_hours": [
      {
        "start_hour": 9,
        "end_hour": 17,
        "days": [1, 2, 3, 4, 5]  // Monday-Friday
      }
    ]
  },
  "figurine_position": {
    "x": 1200,
    "y": 700,
    "anchor": "bottom-right"
  }
}
```

## 🎮 Interacting with Your Companion

### Basic Interactions
- **Drag to move** - Click and drag the skeleton to reposition
- **Pet for encouragement** - Click on the skeleton for a happy bounce
- **Right-click for menu** - Access settings and statistics

### Understanding States
Your skeleton's appearance reflects your focus state:
- 😊 **Solid & Happy** - You're in flow state
- 😴 **Melty & Sleepy** - You might be distracted
- 🎯 **Glowing** - Great focus streak!
- 💤 **Sleeping** - Break time detected

## 📊 Understanding the Metrics

Skelly-Jelly tracks (locally):
- **Keystroke patterns** - Typing rhythm and consistency
- **Window switching** - How often you change apps
- **Mouse movement** - Idle time and click patterns
- **Focus duration** - Time spent in productive states

## 🛠️ Customization

### Adjust Intervention Frequency
```json
{
  "intervention_settings": {
    "min_interval_ms": 1800000  // 30 minutes for less frequent nudges
  }
}
```

### Change Companion Appearance
```json
{
  "figurine_settings": {
    "scale": 1.5,        // Make it bigger
    "opacity": 0.8,      // Make it more transparent
    "color_theme": "purple"  // Different color scheme
  }
}
```

### Set Quiet Hours
```json
{
  "quiet_hours": [
    {
      "start_hour": 12,
      "end_hour": 13,    // No interruptions during lunch
      "days": [1, 2, 3, 4, 5]
    }
  ]
}
```

## 🚦 Troubleshooting

### Common Issues

**Companion not appearing**
- Check system tray for Skelly-Jelly icon
- Verify accessibility permissions granted
- Try `uv run python main.py --debug`

**Too many/few interventions**
- Adjust `min_interval_ms` in config
- Check if you're in configured work hours
- Review intervention history in stats

**High CPU usage**
- Normal during first hour (calibration)
- Should settle to <2% after calibration
- Check `--performance-mode` flag

## 📈 Next Steps

### Enable AI Assistance
1. Download a local LLM model:
   ```bash
   # Download Mistral 7B (recommended)
   curl -L https://huggingface.co/mistral-7b-gguf -o models/mistral-7b.gguf
   ```

2. Update config:
   ```json
   {
     "llm_provider": {
       "type": "llama.cpp",
       "model_path": "./models/mistral-7b.gguf",
       "max_tokens": 100
     }
   }
   ```

### Join the Community
- Report issues: [GitHub Issues]
- Share feedback: [Discord Server]
- Contribute: See [CONTRIBUTING.md]

## 🎯 Tips for ADHD Users

1. **Start small** - Use default settings for a week before customizing
2. **Trust the process** - The AI needs time to learn your patterns
3. **Embrace breaks** - Skelly celebrates rest as much as focus
4. **Adjust freely** - Make it work for YOUR brain

## 🚀 Advanced Features

Once comfortable with basics, explore:
- **Custom intervention messages**
- **Integration with task managers**
- **Detailed analytics dashboard**
- **Multi-device sync** (coming soon)

---

Remember: Skelly-Jelly is here to help, not judge. It's YOUR companion, designed to work with your ADHD brain, not against it. 

*Happy focusing! 🦴✨*