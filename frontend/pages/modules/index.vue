<template>
  <div>
    <div class="mb-6 flex justify-between items-center">
      <h1 class="text-3xl font-bold text-gray-900">Modules</h1>
      <NuxtLink
        to="/modules/upload"
        class="px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700"
      >
        Upload Module
      </NuxtLink>
    </div>
    
    <div v-if="loading" class="text-center py-12">
      <p class="text-gray-600">Loading modules...</p>
    </div>
    
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-md p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>
    
    <div v-else-if="modules.length === 0" class="text-center py-12">
      <p class="text-gray-600 mb-4">No modules uploaded yet</p>
      <NuxtLink
        to="/modules/upload"
        class="inline-block px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700"
      >
        Upload Your First Module
      </NuxtLink>
    </div>
    
    <div v-else class="bg-white shadow-md rounded-lg overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Name
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Hash
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Size
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Created
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          <tr v-for="module in modules" :key="module.id">
            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
              {{ module.name }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 font-mono">
              {{ module.hash.substring(0, 16) }}...
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ formatBytes(module.size_bytes) }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              {{ module.created_at ? formatDate(module.created_at) : 'N/A' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
              <NuxtLink
                :to="`/jobs/create?module=${module.id}`"
                class="text-primary-600 hover:text-primary-700 font-medium"
              >
                Create Job
              </NuxtLink>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Module } from '~/types'

definePageMeta({
  middleware: 'auth'
})

const { fetchModules } = useApi()

const modules = ref<Module[]>([])
const loading = ref(true)
const error = ref('')

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

const formatDate = (dateString: string): string => {
  return new Date(dateString).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

onMounted(async () => {
  try {
    modules.value = await fetchModules()
  } catch (e: any) {
    error.value = e.message || 'Failed to load modules'
  } finally {
    loading.value = false
  }
})
</script>
