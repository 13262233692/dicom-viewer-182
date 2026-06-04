<template>
  <div class="playback-controls" v-if="multiframeInfo">
    <div class="playback-info">
      <span class="frame-info">
        Frame {{ currentFrame + 1 }} / {{ multiframeInfo.number_of_frames }}</span>
      <span class="fps-info" v-if="multiframeInfo.actual_frame_rate > 0">
        {{ actualFps.toFixed(1) }} FPS</span>
      <span class="time-info">{{ formatTime(currentTime) }} / {{ formatTime(totalDuration) }}</span>
    </div>
    
    <div class="playback-slider-container">
      <input
        type="range"
        class="frame-slider"
        :min="0"
        :max="multiframeInfo.number_of_frames - 1"
        v-model.number="currentFrame"
        @input="onSliderChange"
      />
    </div>
    
    <div class="playback-buttons">
      <button class="playback-btn" @click="togglePlay" :title="isPlaying ? 'Pause' : 'Play'">
        {{ isPlaying ? '⏸' : '▶' }}
      </button>
      <button class="playback-btn" @click="goToStart" title="First Frame">
        ⏮
      </button>
      <button class="playback-btn" @click="goToEnd" title="Last Frame">
        ⏭
      </button>
      <button class="playback-btn" @click="stepBackward" title="Previous Frame">
        ◀
      </button>
      <button class="playback-btn" @click="stepForward" title="Next Frame">
        ▶
      </button>
      
      <div class="speed-control">
        <label class="speed-label">Speed:</label>
        <select class="speed-select" v-model.number="playbackSpeed" @change="updateSpeed">
          <option :value="0.25">0.25x</option>
          <option :value="0.5">0.5x</option>
          <option :value="0.75">0.75x</option>
          <option :value="1" selected>1x</option>
          <option :value="1.5">1.5x</option>
          <option :value="2">2x</option>
          <option :value="4">4x</option>
        </select>
      </div>
    </div>
    
    <div class="playback-source" v-if="multiframeInfo.has_frame_time_vector">
      <span class="source-badge">Variable Frame Rate</span>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onUnmounted } from 'vue'

const props = defineProps({
  multiframeInfo: Object,
  frameTimes: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['frame-change', 'playing-state-change'])

const currentFrame = ref(0)
const isPlaying = ref(false)
const playbackSpeed = ref(1.0)
const currentTime = ref(0)

let playInterval = null
let cumulativeTime = 0

const totalDuration = computed(() => {
  if (props.frameTimes.length === 0) return 0
  return props.frameTimes.reduce((sum, t) => sum + t, 0) / 1000
})

const actualFps = computed(() => {
  return props.multiframeInfo?.actual_frame_rate || 0
})

const formatTime = (seconds) => {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  const ms = Math.floor((seconds % 1) * 100)
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`
}

const togglePlay = () => {
  isPlaying.value = !isPlaying.value
  emit('playing-state-change', isPlaying.value)
  if (isPlaying.value) {
    startPlayback()
  } else {
    stopPlayback()
  }
}

const startPlayback = () => {
  stopPlayback()
  playNextFrame()
}

const stopPlayback = () => {
  if (playInterval) {
    clearTimeout(playInterval)
    playInterval = null
  }
}

const playNextFrame = () => {
  if (!isPlaying.value) return
  
  emit('frame-change', currentFrame.value)
  
  const frameTime = props.frameTimes[currentFrame.value] || 33.33
  const adjustedTime = frameTime / playbackSpeed.value
  
  currentFrame.value++
  
  if (currentFrame.value >= props.multiframeInfo.number_of_frames) {
    currentFrame.value = 0
    cumulativeTime = 0
  }
  
  currentTime.value = cumulativeTime / 1000
  
  playInterval = setTimeout(() => {
    cumulativeTime += frameTime
    playNextFrame()
  }, adjustedTime)
}

const goToStart = () => {
  currentFrame.value = 0
  cumulativeTime = 0
  currentTime.value = 0
  emit('frame-change', 0)
}

const goToEnd = () => {
  currentFrame.value = props.multiframeInfo.number_of_frames - 1
  cumulativeTime = totalDuration.value * 1000
  currentTime.value = totalDuration.value
  emit('frame-change', currentFrame.value)
}

const stepBackward = () => {
  if (currentFrame.value > 0) {
    currentFrame.value--
    emit('frame-change', currentFrame.value)
  }
}

const stepForward = () => {
  if (currentFrame.value < props.multiframeInfo.number_of_frames - 1) {
    currentFrame.value++
    emit('frame-change', currentFrame.value)
  }
}

const onSliderChange = () => {
  emit('frame-change', currentFrame.value)
}

const updateSpeed = () => {
  if (isPlaying.value) {
    stopPlayback()
    startPlayback()
  }
}

watch(() => props.multiframeInfo, (newInfo) => {
  if (newInfo) {
    currentFrame.value = 0
    isPlaying.value = false
    stopPlayback()
  }
}, { deep: true })

onUnmounted(() => {
  stopPlayback()
})
</script>

<style scoped>
.playback-controls {
  background: rgba(0, 0, 0, 0.95);
  border-top: 1px solid #2a2a3e;
  padding: 12px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.playback-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #8892b0;
  font-family: monospace;
}

.frame-info {
  color: #64ffda;
  font-weight: 600;
}

.fps-info {
  background: rgba(100, 255, 218, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
  color: #64ffda;
}

.time-info {
  color: #8892b0;
}

.playback-slider-container {
  width: 100%;
}

.frame-slider {
  width: 100%;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: #2a2a3e;
  border-radius: 2px;
  outline: none;
}

.frame-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #64ffda;
  cursor: pointer;
  transition: transform 0.15s ease;
}

.frame-slider::-webkit-slider-thumb:hover {
  transform: scale(1.2);
}

.playback-buttons {
  display: flex;
  align-items: center;
  gap: 8px;
}

.playback-btn {
  background: #2a2a3e;
  border: 1px solid #44475a;
  color: #64ffda;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.15s ease;
  min-width: 40px;
}

.playback-btn:hover {
  background: #64ffda;
  color: #0a0a1a;
  border-color: #64ffda;
}

.speed-control {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.speed-label {
  color: #8892b0;
  font-size: 12px;
}

.speed-select {
  background: #2a2a3e;
  border: 1px solid #44475a;
  color: #ccd6f6;
  padding: 6px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
  outline: none;
}

.speed-select:hover {
  border-color: #64ffda;
}

.playback-source {
  text-align: center;
}

.source-badge {
  background: rgba(255, 193, 7, 0.2);
  color: #ffc107;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 11px;
}
</style>
