<template>
  <div class="mpr-viewport" ref="viewportRef" @wheel="handleWheel">
    <canvas ref="canvasRef"></canvas>
    <div class="plane-label">{{ plane.toUpperCase() }}</div>
    <div v-if="!volumeData" class="empty-state">
      <div class="empty-state-text">No Data</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps({
  plane: String,
  sliceIndex: Number,
  volumeData: Object,
  windowCenter: Number,
  windowWidth: Number
})

const emit = defineEmits(['slice-changed'])

const viewportRef = ref(null)
const canvasRef = ref(null)
const ctx = ref(null)
const scale = ref(1)
const translation = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const lastMousePos = ref({ x: 0, y: 0 })

const resizeCanvas = () => {
  if (!canvasRef.value || !viewportRef.value) return
  const rect = viewportRef.value.getBoundingClientRect()
  canvasRef.value.width = rect.width
  canvasRef.value.height = rect.height
  render()
}

const applyWindow = (value) => {
  const min = props.windowCenter - props.windowWidth / 2
  const max = props.windowCenter + props.windowWidth / 2
  if (value <= min) return 0
  if (value >= max) return 255
  return ((value - min) / props.windowWidth) * 255
}

const getSliceDimensions = () => {
  if (!props.volumeData) return { width: 0, height: 0 }
  const dims = props.volumeData.dimensions
  switch (props.plane) {
    case 'axial': return { width: dims[0], height: dims[1] }
    case 'sagittal': return { width: dims[1], height: dims[2] }
    case 'coronal': return { width: dims[0], height: dims[2] }
    default: return { width: 0, height: 0 }
  }
}

const getMaxSlice = () => {
  if (!props.volumeData) return 0
  const dims = props.volumeData.dimensions
  switch (props.plane) {
    case 'axial': return dims[2] - 1
    case 'sagittal': return dims[0] - 1
    case 'coronal': return dims[1] - 1
    default: return 0
  }
}

const getVoxel = (x, y, z) => {
  if (!props.volumeData) return 0
  const dims = props.volumeData.dimensions
  const data = props.volumeData.voxel_data
  if (x < 0 || x >= dims[0] || y < 0 || y >= dims[1] || z < 0 || z >= dims[2]) return 0
  const idx = z * dims[0] * dims[1] + y * dims[0] + x
  return data[idx] || 0
}

const render = () => {
  if (!ctx.value || !canvasRef.value || !props.volumeData) return
  
  const canvas = canvasRef.value
  const { width, height } = getSliceDimensions()
  const slice = props.sliceIndex
  
  ctx.value.fillStyle = '#000'
  ctx.value.fillRect(0, 0, canvas.width, canvas.height)
  
  if (width === 0 || height === 0) return
  
  const imageAspect = width / height
  const canvasAspect = canvas.width / canvas.height
  
  let renderWidth, renderHeight
  if (imageAspect > canvasAspect) {
    renderWidth = canvas.width * scale.value
    renderHeight = renderWidth / imageAspect
  } else {
    renderHeight = canvas.height * scale.value
    renderWidth = renderHeight * imageAspect
  }
  
  const offsetX = (canvas.width - renderWidth) / 2 + translation.value.x
  const offsetY = (canvas.height - renderHeight) / 2 + translation.value.y
  
  const imageData = ctx.value.createImageData(renderWidth, renderHeight)
  const data = imageData.data
  
  for (let py = 0; py < renderHeight; py++) {
    for (let px = 0; px < renderWidth; px++) {
      let value = 0
      
      switch (props.plane) {
        case 'axial': {
          const vx = Math.floor((px / renderWidth) * width)
          const vy = Math.floor((py / renderHeight) * height)
          value = getVoxel(vx, vy, slice)
          break
        }
        case 'sagittal': {
          const vx = slice
          const vy = Math.floor((px / renderWidth) * width)
          const vz = Math.floor((py / renderHeight) * height)
          value = getVoxel(vx, vy, vz)
          break
        }
        case 'coronal': {
          const vx = Math.floor((px / renderWidth) * width)
          const vy = slice
          const vz = Math.floor((py / renderHeight) * height)
          value = getVoxel(vx, vy, vz)
          break
        }
      }
      
      const pixel = applyWindow(value)
      const dstIdx = (py * renderWidth + px) * 4
      data[dstIdx] = pixel
      data[dstIdx + 1] = pixel
      data[dstIdx + 2] = pixel
      data[dstIdx + 3] = 255
    }
  }
  
  ctx.value.putImageData(imageData, offsetX, offsetY)
}

const handleMouseDown = (e) => {
  isDragging.value = true
  lastMousePos.value = { x: e.clientX, y: e.clientY }
}

const handleMouseMove = (e) => {
  if (isDragging.value) {
    translation.value.x += e.clientX - lastMousePos.value.x
    translation.value.y += e.clientY - lastMousePos.value.y
    lastMousePos.value = { x: e.clientX, y: e.clientY }
    render()
  }
}

const handleMouseUp = () => {
  isDragging.value = false
}

const handleWheel = (e) => {
  e.preventDefault()
  e.stopPropagation()
  
  const maxSlice = getMaxSlice()
  const direction = e.deltaY > 0 ? 1 : -1
  const newSlice = Math.max(0, Math.min(maxSlice, props.sliceIndex + direction))
  
  if (newSlice !== props.sliceIndex) {
    emit('slice-changed', newSlice)
  }
}

watch([() => props.sliceIndex, () => props.volumeData, () => props.windowCenter, () => props.windowWidth], () => {
  render()
})

onMounted(() => {
  ctx.value = canvasRef.value?.getContext('2d')
  window.addEventListener('resize', resizeCanvas)
  resizeCanvas()
  
  if (viewportRef.value) {
    viewportRef.value.addEventListener('mousedown', handleMouseDown)
    viewportRef.value.addEventListener('mousemove', handleMouseMove)
    viewportRef.value.addEventListener('mouseup', handleMouseUp)
    viewportRef.value.addEventListener('mouseleave', handleMouseUp)
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', resizeCanvas)
  if (viewportRef.value) {
    viewportRef.value.removeEventListener('mousedown', handleMouseDown)
    viewportRef.value.removeEventListener('mousemove', handleMouseMove)
    viewportRef.value.removeEventListener('mouseup', handleMouseUp)
    viewportRef.value.removeEventListener('mouseleave', handleMouseUp)
  }
})
</script>

<style scoped>
.mpr-viewport {
  width: 100%;
  height: 100%;
  position: relative;
  background: #000;
  cursor: grab;
}

.mpr-viewport:active {
  cursor: grabbing;
}

.mpr-viewport canvas {
  width: 100%;
  height: 100%;
}

.plane-label {
  position: absolute;
  top: 8px;
  left: 8px;
  background: rgba(0, 0, 0, 0.7);
  color: #e94560;
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  pointer-events: none;
}
</style>
