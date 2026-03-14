<template>
  <div class="min-h-screen bg-gray-50">
    <nav class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex items-center">
            <NuxtLink to="/" class="text-2xl font-bold text-primary-600">
              DRASM
            </NuxtLink>
            <div v-if="isAuthenticated" class="ml-10 flex space-x-4">
              <NuxtLink
                to="/modules"
                class="px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100"
              >
                Modules
              </NuxtLink>
              <NuxtLink
                to="/jobs"
                class="px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100"
              >
                Jobs
              </NuxtLink>
               <NuxtLink
                to="/pricing"
                class="px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100"
              >
                Pricing
              </NuxtLink>
               <NuxtLink
                to="/docs"
                class="px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100"
              >
                Documentation
              </NuxtLink>
              
            </div>
          </div>
          <div class="flex items-center space-x-4">
            <template v-if="isAuthenticated">
              <span v-if="user?.email" class="text-sm text-gray-600">{{ user.email }}</span>
              <button
                @click="handleLogout"
                class="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-md hover:bg-primary-700"
              >
                Logout
              </button>
            </template>
            <template v-else>
              <NuxtLink
                to="/login"
                class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
              >
                Login
              </NuxtLink>
              <NuxtLink
                to="/register"
                class="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-md hover:bg-primary-700"
              >
                Register
              </NuxtLink>
            </template>
          </div>
        </div>
      </div>
    </nav>

    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
const { isAuthenticated, user, logout } = useAuth()

const handleLogout = () => {
  logout()
}
</script>
