<template>
  <div class="app-container">
    <Toolbar
      :activeTool="activeTool"
      @tool-change="handleToolChange"
      @open-file="openFile"
      @open-directory="openDirectory"
      @open-sr="openSRFile"
    />
    
    <div class="main-content">
      <Sidebar
        :imageInfo="imageInfo"
        :volumeInfo="volumeInfo"
        :multiframeInfo="multiframeInfo"
        :windowCenter="windowCenter"
        :windowWidth="windowWidth"
        :measurements="measurements"
        :mprSlices="mprSlices"
        :srInfo="srInfoData"
        @window-change="handleWindowChange"
        @mpr-slice-change="handleMprSliceChange"
        @clear-measurements="clearMeasurements"
        @open-sr-panel="showSRPanel = true"
      />
      
      <div class="viewport-area">
        <div v-if="viewMode === '2d'" class="viewport-container with-playback">
          <Viewport2D
            ref="viewport2d"
            :imageData="currentDisplayImage"
            :activeTool="activeTool"
            :windowCenter="windowCenter"
            :windowWidth="windowWidth"
            :srHighlights="srHighlights"
            @measurement-added="handleMeasurementAdded"
          />
          <PlaybackControls
            v-if="multiframeInfo"
            :multiframeInfo="multiframeInfo"
            :frameTimes="frameTimes"
            @frame-change="handleFrameChange"
            @playing-state-change="handlePlayingStateChange"
          />
        </div>

        <div v-if="showSRPanel && srDocument" class="sr-panel">
          <div class="sr-panel-header">
            <span>SR Report</span>
            <button class="sr-close-btn" @click="showSRPanel = false">✕</button>
          </div>
          <SRReport
            :srDocument="srDocument"
            @navigate-to-image="handleSRNavigateToImage"
            @highlight-sr-item="handleSRHighlight"
          />
        </div>
        
        <div v-else-if="viewMode === 'mpr'" class="grid-layout grid-2x2">
          <div class="grid-cell">
            <MPRViewport
              :plane="'axial'"
              :sliceIndex="mprSlices.axial"
              :volumeData="volumeData"
              :windowCenter="windowCenter"
              :windowWidth="windowWidth"
              @slice-changed="(idx) => handleMprSliceChange('axial', idx)"
            />
          </div>
          <div class="grid-cell">
            <MPRViewport
              :plane="'sagittal'"
              :sliceIndex="mprSlices.sagittal"
              :volumeData="volumeData"
              :windowCenter="windowCenter"
              :windowWidth="windowWidth"
              @slice-changed="(idx) => handleMprSliceChange('sagittal', idx)"
            />
          </div>
          <div class="grid-cell">
            <MPRViewport
              :plane="'coronal'"
              :sliceIndex="mprSlices.coronal"
              :volumeData="volumeData"
              :windowCenter="windowCenter"
              :windowWidth="windowWidth"
              @slice-changed="(idx) => handleMprSliceChange('coronal', idx)"
            />
          </div>
          <div class="grid-cell">
            <VolumeRenderView
              v-if="volumeData"
              :volumeData="volumeData"
              :windowCenter="windowCenter"
              :windowWidth="windowWidth"
            />
            <div v-else class="empty-state">
              <div class="empty-state-icon">📊</div>
              <div class="empty-state-text">3D Volume Rendering</div>
              <div class="empty-state-hint">Load DICOM series first</div>
            </div>
          </div>
        </div>
        
        <div v-else-if="viewMode === '3d'" class="viewport-container">
          <VolumeRenderView
            v-if="volumeData"
            :volumeData="volumeData"
            :windowCenter="windowCenter"
            :windowWidth="windowWidth"
          />
          <div v-else class="empty-state">
            <div class="empty-state-icon">🧊</div>
            <div class="empty-state-text">No Volume Loaded</div>
            <div class="empty-state-hint">Open a DICOM directory to view 3D rendering</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import Toolbar from './components/Toolbar.vue'
import Sidebar from './components/Sidebar.vue'
import Viewport2D from './components/Viewport2D.vue'
import MPRViewport from './components/MPRViewport.vue'
import VolumeRenderView from './components/VolumeRenderView.vue'
import PlaybackControls from './components/PlaybackControls.vue'
import SRReport from './components/SRReport.vue'

const activeTool = ref('pan')
const viewMode = ref('2d')
const imageData = ref(null)
const volumeData = ref(null)
const imageInfo = ref(null)
const volumeInfo = ref(null)
const windowCenter = ref(40)
const windowWidth = ref(400)
const measurements = ref([])
const viewport2d = ref(null)

const multiframeInfo = ref(null)
const frameTimes = ref([])
const currentFrameIndex = ref(0)
const framePixelData = ref([])
const isPlaying = ref(false)

const srDocument = ref(null)
const srHighlights = ref([])
const showSRPanel = ref(false)

const mprSlices = reactive({
  axial: 0,
  sagittal: 0,
  coronal: 0
})

const currentDisplayImage = computed(() => {
  if (!multiframeInfo.value || framePixelData.value.length === 0) {
    return imageData.value
  }
  
  if (currentFrameIndex.value < framePixelData.value.length && imageData.value) {
    return {
      ...imageData.value,
      pixel_data: framePixelData.value[currentFrameIndex.value]
    }
  }
  
  return imageData.value
})

const srInfoData = computed(() => {
  if (!srDocument.value) return null
  return {
    title: srDocument.value.title || srDocument.value.completion_flag,
    conclusionCount: srDocument.value.conclusions ? srDocument.value.conclusions.length : 0,
    measurementCount: srDocument.value.measurements ? srDocument.value.measurements.length : 0,
    imageRefCount: srDocument.value.image_references ? srDocument.value.image_references.length : 0,
    panelOpen: showSRPanel.value
  }
})

const handleToolChange = (tool) => {
  activeTool.value = tool
  if (tool === 'mpr') {
    viewMode.value = 'mpr'
  } else if (tool === '3d') {
    viewMode.value = '3d'
  } else {
    viewMode.value = '2d'
  }
}

const openFile = async () => {
  const selected = await open({
    filters: [{
      name: 'DICOM',
      extensions: ['dcm', 'dicom', '']
    }],
    multiple: false
  })
  
  if (selected) {
    try {
      const result = await invoke('load_dicom_file', {
        request: { path: selected }
      })
      imageData.value = result
      imageInfo.value = {
        patientName: result.patient_name,
        patientId: result.patient_id,
        studyDate: result.study_date,
        modality: result.modality,
        seriesDescription: result.series_description,
        width: result.width,
        height: result.height,
        spacing: result.spacing,
        isMultiframe: result.is_multiframe,
        numberOfFrames: result.number_of_frames
      }
      windowCenter.value = result.window_center
      windowWidth.value = result.window_width
      viewMode.value = '2d'
      activeTool.value = 'pan'
      
      if (result.modality === 'SR') {
        try {
          const srResult = await invoke('get_sr_document')
          if (srResult) {
            srDocument.value = srResult
            showSRPanel.value = true
            if (srResult.measurements && srResult.measurements.length > 0) {
              const highlightItems = srResult.measurements
                .filter(m => m.spatial_coordinates)
                .map(m => ({
                  name: m.name,
                  spatial_coordinates: m.spatial_coordinates
                }))
              srHighlights.value = highlightItems
            }
          }
        } catch (e) {
          console.error('Failed to load SR data:', e)
        }
      } else {
        srDocument.value = null
        srHighlights.value = []
        showSRPanel.value = false
      }
      
      if (result.is_multiframe) {
        const mfInfo = await invoke('get_multiframe_info')
        multiframeInfo.value = mfInfo
        
        const mfData = await invoke('get_multiframe_data')
        if (mfData) {
          framePixelData.value = mfData.frames || []
          frameTimes.value = mfData.frame_times || []
        }
        
        currentFrameIndex.value = 0
      } else {
        multiframeInfo.value = null
        framePixelData.value = []
        frameTimes.value = []
        currentFrameIndex.value = 0
      }
    } catch (error) {
      console.error('Failed to load DICOM file:', error)
    }
  }
}

const handleFrameChange = async (frameIndex) => {
  if (frameIndex >= 0 && frameIndex < framePixelData.value.length) {
    currentFrameIndex.value = frameIndex
  } else if (multiframeInfo.value) {
    try {
      const frameData = await invoke('get_frame', {
        request: { frame_index: frameIndex }
      })
      if (frameData && framePixelData.value.length > frameIndex) {
        framePixelData.value[frameIndex] = frameData
      }
      currentFrameIndex.value = frameIndex
    } catch (error) {
      console.error('Failed to get frame:', error)
    }
  }
}

const handlePlayingStateChange = (playing) => {
  isPlaying.value = playing
}

const handleSRNavigateToImage = (imageRef) => {
  console.log('Navigate to SR image reference:', imageRef.sop_instance_uid)
}

const handleSRHighlight = (item) => {
  const highlights = []
  
  if (item.type === 'measurement' && item.data) {
    if (item.data.spatial_coordinates) {
      highlights.push({
        name: item.data.name,
        spatial_coordinates: item.data.spatial_coordinates
      })
    }
  } else if (item.type === 'conclusion' && item.data) {
    if (item.data.image_reference) {
      console.log('SR conclusion references image:', item.data.image_reference.sop_instance_uid)
    }
  } else if (item.type === 'node' && item.data) {
    if (item.data.spatial_coordinates) {
      highlights.push({
        name: item.data.concept_name,
        spatial_coordinates: item.data.spatial_coordinates
      })
    }
  }
  
  srHighlights.value = highlights
}

const openSRFile = async () => {
  const selected = await open({
    filters: [{
      name: 'DICOM SR',
      extensions: ['dcm', 'dicom', '']
    }],
    multiple: false
  })
  
  if (selected) {
    try {
      const result = await invoke('load_sr_file', {
        request: { path: selected }
      })
      srDocument.value = result
      showSRPanel.value = true
      
      if (result.measurements && result.measurements.length > 0) {
        const highlightItems = result.measurements
          .filter(m => m.spatial_coordinates)
          .map(m => ({
            name: m.name,
            spatial_coordinates: m.spatial_coordinates
          }))
        srHighlights.value = highlightItems
      }
    } catch (error) {
      console.error('Failed to load SR file:', error)
    }
  }
}

const openDirectory = async () => {
  const selected = await open({
    directory: true
  })
  
  if (selected) {
    try {
      const seriesInfo = await invoke('load_dicom_directory', {
        request: { directory: selected }
      })
      
      const volInfo = await invoke('get_volume_info')
      volumeInfo.value = volInfo
      
      const volData = await invoke('get_volume_data')
      volumeData.value = volData
      
      mprSlices.axial = Math.floor(volInfo.depth / 2)
      mprSlices.sagittal = Math.floor(volInfo.width / 2)
      mprSlices.coronal = Math.floor(volInfo.height / 2)
      
      if (seriesInfo && seriesInfo.length > 0) {
        imageInfo.value = {
          patientName: seriesInfo[0].patient_name,
          studyDate: seriesInfo[0].study_date,
          modality: seriesInfo[0].modality,
          seriesDescription: seriesInfo[0].description,
          numSlices: seriesInfo[0].num_slices
        }
      }
      
      windowCenter.value = volInfo.window_center
      windowWidth.value = volInfo.window_width
      viewMode.value = 'mpr'
      activeTool.value = 'mpr'
    } catch (error) {
      console.error('Failed to load DICOM directory:', error)
    }
  }
}

const handleWindowChange = (center, width) => {
  windowCenter.value = center
  windowWidth.value = width
}

const handleMprSliceChange = (plane, index) => {
  mprSlices[plane] = index
}

const handleMeasurementAdded = (measurement) => {
  measurements.value.push(measurement)
}

const clearMeasurements = () => {
  measurements.value = []
  if (viewport2d.value) {
    viewport2d.value.clearMeasurements()
  }
}
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.viewport-area {
  flex: 1;
  display: flex;
  background: #000;
  overflow: hidden;
}

.viewport-container {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
}

.with-playback {
  height: 100%;
}

.sr-panel {
  width: 320px;
  min-width: 320px;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sr-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border);
}

.sr-close-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 6px;
  border-radius: 4px;
  transition: all 0.15s;
}

.sr-close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--accent);
}
</style>
