<template>
  <div class="volume-render-view" ref="containerRef">
    <div v-if="!initialized" class="loading-overlay">
      <div class="loading-text">Initializing 3D Renderer...</div>
    </div>
    <div class="volume-controls">
      <button @click="setPreset('bone')">Bone</button>
      <button @click="setPreset('skin')">Skin</button>
      <button @click="setPreset('muscle')">Muscle</button>
      <button @click="toggleCropping">Crop</button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps({
  volumeData: Object,
  windowCenter: Number,
  windowWidth: Number
})

const containerRef = ref(null)
const initialized = ref(false)
const isCropping = ref(false)

let renderer = null
let renderWindow = null
let interactor = null
let volumeMapper = null
let volume = null
let imageData = null
let opacityTransferFunction = null
let colorTransferFunction = null
let volumeProperty = null
let boxWidget = null
let cropFilter = null

const presets = {
  bone: {
    opacityPoints: [
      [-1000, 0],
      [-100, 0],
      [100, 0.2],
      [300, 0.5],
      [500, 0.8],
      [1000, 1]
    ],
    colorPoints: [
      [-1000, [0, 0, 0]],
      [0, [0.8, 0.6, 0.4]],
      [300, [1, 0.9, 0.7]],
      [1000, [1, 1, 1]]
    ]
  },
  skin: {
    opacityPoints: [
      [-1000, 0],
      [-200, 0],
      [0, 0.05],
      [50, 0.2],
      [100, 0.4],
      [500, 0.1]
    ],
    colorPoints: [
      [-1000, [0, 0, 0]],
      [0, [1, 0.75, 0.65]],
      [500, [1, 0.85, 0.8]]
    ]
  },
  muscle: {
    opacityPoints: [
      [-1000, 0],
      [-50, 0],
      [20, 0.3],
      [60, 0.5],
      [100, 0.3],
      [500, 0]
    ],
    colorPoints: [
      [-1000, [0, 0, 0]],
      [0, [0.9, 0.5, 0.4]],
      [100, [0.7, 0.3, 0.25]],
      [500, [0.5, 0.2, 0.15]]
    ]
  }
}

const initVTK = async () => {
  try {
    const vtk = await import('@kitware/vtk.js')
    
    const vtkFullScreenRenderWindow = vtk.Rendering.Misc.vtkFullScreenRenderWindow
    const vtkImageData = vtk.Common.DataModel.vtkImageData
    const vtkDataArray = vtk.Common.Core.vtkDataArray
    const vtkVolume = vtk.Rendering.Core.vtkVolume
    const vtkVolumeMapper = vtk.Rendering.Core.vtkVolumeMapper
    const vtkColorTransferFunction = vtk.Rendering.Core.vtkColorTransferFunction
    const vtkPiecewiseFunction = vtk.Common.DataModel.vtkPiecewiseFunction
    const vtkVolumeProperty = vtk.Rendering.Core.vtkVolumeProperty
    const vtkBoxWidget = vtk.Widgets.Widgets3D.vtkBoxWidget
    const vtkImageCroppingRegionsWidget = vtk.Widgets.Widgets3D.vtkImageCroppingRegionsWidget
    
    const container = containerRef.value
    if (!container) return
    
    const fullScreenRenderer = vtkFullScreenRenderWindow.newInstance({
      background: [0, 0, 0],
      container,
      containerStyle: {}
    })
    
    renderer = fullScreenRenderer.getRenderer()
    renderWindow = fullScreenRenderer.getRenderWindow()
    interactor = renderWindow.getInteractor()
    
    volumeMapper = vtkVolumeMapper.newInstance()
    volume = vtkVolume.newInstance()
    volume.setMapper(volumeMapper)
    
    colorTransferFunction = vtkColorTransferFunction.newInstance()
    opacityTransferFunction = vtkPiecewiseFunction.newInstance()
    
    volumeProperty = vtkVolumeProperty.newInstance()
    volumeProperty.setColor(colorTransferFunction)
    volumeProperty.setScalarOpacity(opacityTransferFunction)
    volumeProperty.setInterpolationTypeToLinear()
    volumeProperty.setShade(true)
    volumeProperty.setAmbient(0.1)
    volumeProperty.setDiffuse(0.9)
    volumeProperty.setSpecular(0.2)
    volumeProperty.setSpecularPower(10)
    
    volume.setProperty(volumeProperty)
    
    imageData = vtkImageData.newInstance()
    
    renderer.addVolume(volume)
    renderer.resetCamera()
    
    initialized.value = true
    
    if (props.volumeData) {
      updateVolume()
    }
  } catch (error) {
    console.error('Failed to initialize VTK:', error)
  }
}

const updateVolume = () => {
  if (!initialized.value || !props.volumeData || !imageData) return
  
  const dims = props.volumeData.dimensions
  const spacing = props.volumeData.spacing
  const data = props.volumeData.voxel_data
  
  imageData.setDimensions(dims[0], dims[1], dims[2])
  imageData.setSpacing(spacing[0], spacing[1], spacing[2])
  imageData.setOrigin(0, 0, 0)
  
  const scalarsData = new Uint16Array(data)
  const vtkDataArray = require('@kitware/vtk.js/Common/Core/vtkDataArray').default
  const scalars = vtkDataArray.newInstance({
    numberOfComponents: 1,
    values: scalarsData,
    dataType: vtkDataArray.VtkDataTypes.UINT16
  })
  
  imageData.getPointData().setScalars(scalars)
  
  volumeMapper.setInputData(imageData)
  
  setPreset('bone')
  
  renderer.resetCamera()
  renderWindow.render()
}

const setPreset = (presetName) => {
  const preset = presets[presetName]
  if (!preset || !colorTransferFunction || !opacityTransferFunction) return
  
  colorTransferFunction.removeAllPoints()
  for (const [value, color] of preset.colorPoints) {
    colorTransferFunction.addRGBPoint(value, color[0], color[1], color[2])
  }
  
  opacityTransferFunction.removeAllPoints()
  for (const [value, opacity] of preset.opacityPoints) {
    opacityTransferFunction.addPoint(value, opacity)
  }
  
  if (renderWindow) {
    renderWindow.render()
  }
}

const toggleCropping = () => {
  isCropping.value = !isCropping.value
  // Box widget implementation would go here
}

watch(() => props.volumeData, () => {
  if (initialized.value) {
    updateVolume()
  }
})

onMounted(() => {
  initVTK()
})

onUnmounted(() => {
  if (renderWindow) {
    renderWindow.delete()
  }
  if (interactor) {
    interactor.delete()
  }
  if (renderer) {
    renderer.delete()
  }
})
</script>

<style scoped>
.volume-render-view {
  width: 100%;
  height: 100%;
  position: relative;
  background: #000;
  overflow: hidden;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
  z-index: 10;
}

.loading-text {
  color: #fff;
  font-size: 14px;
}

.volume-controls {
  position: absolute;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;
  background: rgba(0, 0, 0, 0.7);
  padding: 8px 12px;
  border-radius: 8px;
  z-index: 5;
}

.volume-controls button {
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.volume-controls button:hover {
  background: var(--accent);
  border-color: var(--accent);
}
</style>
