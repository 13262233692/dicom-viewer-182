<template>
  <div class="toolbar">
    <button class="toolbar-btn" @click="$emit('open-file')">
      <span>📄</span>
      <span>Open File</span>
    </button>
    <button class="toolbar-btn" @click="$emit('open-directory')">
      <span>📁</span>
      <span>Open Series</span>
    </button>
    <button class="toolbar-btn" @click="$emit('open-sr')">
      <span>📋</span>
      <span>SR Report</span>
    </button>
    
    <div style="width: 1px; background: var(--border); margin: 0 8px;"></div>
    
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'pan' }"
      @click="$emit('tool-change', 'pan')"
    >
      <span>✋</span>
      <span>Pan</span>
    </button>
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'zoom' }"
      @click="$emit('tool-change', 'zoom')"
    >
      <span>🔍</span>
      <span>Zoom</span>
    </button>
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'window' }"
      @click="$emit('tool-change', 'window')"
    >
      <span>🎚️</span>
      <span>Window</span>
    </button>
    
    <div style="width: 1px; background: var(--border); margin: 0 8px;"></div>
    
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'length' }"
      @click="$emit('tool-change', 'length')"
    >
      <span>📏</span>
      <span>Length</span>
    </button>
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'angle' }"
      @click="$emit('tool-change', 'angle')"
    >
      <span>📐</span>
      <span>Angle</span>
    </button>
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'area' }"
      @click="$emit('tool-change', 'area')"
    >
      <span>⬛</span>
      <span>Area</span>
    </button>
    
    <div style="width: 1px; background: var(--border); margin: 0 8px;"></div>
    
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === 'mpr' }"
      @click="$emit('tool-change', 'mpr')"
    >
      <span>🔲</span>
      <span>MPR</span>
    </button>
    <button 
      class="toolbar-btn" 
      :class="{ active: activeTool === '3d' }"
      @click="$emit('tool-change', '3d')"
    >
      <span>🧊</span>
      <span>3D Volume</span>
    </button>
  </div>
</template>

<script setup>
defineProps({
  activeTool: String
})

defineEmits(['tool-change', 'open-file', 'open-directory', 'open-sr'])
</script>
