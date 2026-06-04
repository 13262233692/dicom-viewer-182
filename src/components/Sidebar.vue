<template>
  <div class="sidebar">
    <div class="sidebar-section">
      <div class="sidebar-title">Patient Info</div>
      <div v-if="imageInfo" class="info-content">
        <div class="info-item">
          <span class="info-label">Name:</span>
          <span class="info-value">{{ imageInfo.patientName || 'N/A' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Date:</span>
          <span class="info-value">{{ imageInfo.studyDate || 'N/A' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Modality:</span>
          <span class="info-value">{{ imageInfo.modality || 'N/A' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Series:</span>
          <span class="info-value">{{ imageInfo.seriesDescription || 'N/A' }}</span>
        </div>
        <div v-if="imageInfo.isMultiframe" class="info-item">
          <span class="info-label">Frames:</span>
          <span class="info-value highlight">{{ imageInfo.numberOfFrames }}</span>
        </div>
      </div>
      <div v-else class="empty-info">No data loaded</div>
    </div>

    <div v-if="multiframeInfo" class="sidebar-section">
      <div class="sidebar-title">Playback Info</div>
      <div class="info-content">
        <div class="info-item">
          <span class="info-label">Frame Rate:</span>
          <span class="info-value highlight">{{ multiframeInfo.actual_frame_rate.toFixed(2) }} FPS</span>
        </div>
        <div class="info-item">
          <span class="info-label">Frame Time:</span>
          <span class="info-value">{{ multiframeInfo.frame_time.toFixed(2) }} ms</span>
        </div>
        <div class="info-item" v-if="multiframeInfo.recommended_frame_rate > 0">
          <span class="info-label">Recommended:</span>
          <span class="info-value">{{ multiframeInfo.recommended_frame_rate.toFixed(1) }} FPS</span>
        </div>
        <div class="info-item" v-if="multiframeInfo.has_frame_time_vector">
          <span class="info-label">Timing:</span>
          <span class="info-value warning">Variable</span>
        </div>
        <div class="info-item" v-else>
          <span class="info-label">Timing:</span>
          <span class="info-value">Fixed</span>
        </div>
      </div>
    </div>

    <div class="sidebar-section">
      <div class="sidebar-title">Window Level</div>
      <div class="window-controls">
        <div class="control-row">
          <label>Center: {{ windowCenter }}</label>
          <input 
            type="range" 
            :value="windowCenter" 
            min="-1000" 
            max="1000" 
            step="1"
            @input="updateWindowCenter"
          />
        </div>
        <div class="control-row">
          <label>Width: {{ windowWidth }}</label>
          <input 
            type="range" 
            :value="windowWidth" 
            min="1" 
            max="2000" 
            step="1"
            @input="updateWindowWidth"
          />
        </div>
        <div class="preset-buttons">
          <button @click="setPreset('lung')">Lung</button>
          <button @click="setPreset('bone')">Bone</button>
          <button @click="setPreset('brain')">Brain</button>
          <button @click="setPreset('abdomen')">Abdomen</button>
        </div>
      </div>
    </div>

    <div v-if="volumeInfo" class="sidebar-section">
      <div class="sidebar-title">MPR Navigation</div>
      <div class="mpr-controls">
        <div class="control-row">
          <label>Axial: {{ mprSlices.axial + 1 }} / {{ volumeInfo.depth }}</label>
          <input 
            type="range" 
            :value="mprSlices.axial" 
            min="0" 
            :max="volumeInfo.depth - 1" 
            step="1"
            @input="$emit('mpr-slice-change', 'axial', parseInt($event.target.value))"
          />
        </div>
        <div class="control-row">
          <label>Sagittal: {{ mprSlices.sagittal + 1 }} / {{ volumeInfo.width }}</label>
          <input 
            type="range" 
            :value="mprSlices.sagittal" 
            min="0" 
            :max="volumeInfo.width - 1" 
            step="1"
            @input="$emit('mpr-slice-change', 'sagittal', parseInt($event.target.value))"
          />
        </div>
        <div class="control-row">
          <label>Coronal: {{ mprSlices.coronal + 1 }} / {{ volumeInfo.height }}</label>
          <input 
            type="range" 
            :value="mprSlices.coronal" 
            min="0" 
            :max="volumeInfo.height - 1" 
            step="1"
            @input="$emit('mpr-slice-change', 'coronal', parseInt($event.target.value))"
          />
        </div>
      </div>
    </div>

    <div class="sidebar-section">
      <div class="sidebar-title">
        Measurements
        <button v-if="measurements.length > 0" class="clear-btn" @click="$emit('clear-measurements')">Clear</button>
      </div>
      <div v-if="measurements.length > 0" class="measurements-list">
        <div v-for="(m, idx) in measurements" :key="idx" class="measurement-item">
          <span class="measurement-type">{{ m.type }}</span>
          <span class="measurement-value">{{ formatMeasurement(m) }}</span>
        </div>
      </div>
      <div v-else class="empty-info">No measurements</div>
    </div>

    <div v-if="volumeInfo" class="sidebar-section">
      <div class="sidebar-title">Volume Info</div>
      <div class="info-content">
        <div class="info-item">
          <span class="info-label">Dimensions:</span>
          <span class="info-value">{{ volumeInfo.width }} x {{ volumeInfo.height }} x {{ volumeInfo.depth }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Spacing:</span>
          <span class="info-value">{{ volumeInfo.spacingX.toFixed(2) }} x {{ volumeInfo.spacingY.toFixed(2) }} x {{ volumeInfo.spacingZ.toFixed(2) }} mm</span>
        </div>
      </div>
    </div>

    <div v-if="srInfo" class="sidebar-section">
      <div class="sidebar-title">SR Report</div>
      <div class="info-content">
        <div class="info-item">
          <span class="info-label">Title:</span>
          <span class="info-value">{{ srInfo.title || 'N/A' }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Conclusion:</span>
          <span class="info-value highlight">{{ srInfo.conclusionCount }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Measurements:</span>
          <span class="info-value highlight">{{ srInfo.measurementCount }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">References:</span>
          <span class="info-value">{{ srInfo.imageRefCount }}</span>
        </div>
        <button v-if="!srInfo.panelOpen" class="sr-open-btn" @click="$emit('open-sr-panel')">Open Report</button>
      </div>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  imageInfo: Object,
  volumeInfo: Object,
  multiframeInfo: Object,
  windowCenter: Number,
  windowWidth: Number,
  measurements: Array,
  mprSlices: Object,
  srInfo: Object
})

const emit = defineEmits(['window-change', 'mpr-slice-change', 'clear-measurements', 'open-sr-panel'])

const updateWindowCenter = (e) => {
  emit('window-change', parseInt(e.target.value), props.windowWidth)
}

const updateWindowWidth = (e) => {
  emit('window-change', props.windowCenter, parseInt(e.target.value))
}

const presets = {
  lung: { center: -600, width: 1500 },
  bone: { center: 300, width: 1500 },
  brain: { center: 40, width: 80 },
  abdomen: { center: 50, width: 350 }
}

const setPreset = (preset) => {
  const { center, width } = presets[preset]
  emit('window-change', center, width)
}

const formatMeasurement = (m) => {
  switch (m.type) {
    case 'Length':
      return `${m.value.toFixed(2)} mm`
    case 'Angle':
      return `${m.value.toFixed(1)}°`
    case 'Area':
      return `${m.value.toFixed(2)} mm²`
    default:
      return m.value.toFixed(2)
  }
}
</script>

<style scoped>
.info-content {
  font-size: 13px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 6px;
}

.info-label {
  color: var(--text-secondary);
}

.info-value {
  color: var(--text-primary);
  text-align: right;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.info-value.highlight {
  color: var(--accent);
  font-weight: 600;
}

.info-value.warning {
  color: #ffc107;
  font-weight: 600;
}

.empty-info {
  color: var(--text-secondary);
  font-size: 13px;
  font-style: italic;
}

.window-controls, .mpr-controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.control-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.control-row label {
  font-size: 12px;
  color: var(--text-secondary);
}

.preset-buttons {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.preset-buttons button {
  padding: 6px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.preset-buttons button:hover {
  background: var(--accent);
  border-color: var(--accent);
}

.sidebar-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.clear-btn {
  padding: 3px 8px;
  background: transparent;
  border: 1px solid var(--accent);
  border-radius: 4px;
  color: var(--accent);
  cursor: pointer;
  font-size: 11px;
  transition: all 0.2s;
}

.clear-btn:hover {
  background: var(--accent);
  color: white;
}

.measurements-list {
  max-height: 200px;
  overflow-y: auto;
}

.sr-open-btn {
  margin-top: 8px;
  padding: 6px 12px;
  background: rgba(0, 255, 200, 0.1);
  border: 1px solid rgba(0, 255, 200, 0.3);
  border-radius: 4px;
  color: #64ffda;
  cursor: pointer;
  font-size: 12px;
  width: 100%;
  transition: all 0.2s;
}

.sr-open-btn:hover {
  background: rgba(0, 255, 200, 0.2);
  border-color: #64ffda;
}
</style>
