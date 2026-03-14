<template>
  <div class="max-w-md mx-auto">
    <div class="bg-white rounded-lg shadow-md p-8">
      <h2 class="text-2xl font-bold text-gray-900 mb-6">Login</h2>
      
      <form @submit.prevent="handleLogin" class="space-y-4">
        <div>
          <label for="email" class="block text-sm font-medium text-gray-700 mb-1">
            Email
          </label>
          <input
            id="email"
            v-model="email"
            type="email"
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
            placeholder="you@example.com"
          />
        </div>
        
        <div>
          <label for="password" class="block text-sm font-medium text-gray-700 mb-1">
            Password
          </label>
          <input
            id="password"
            v-model="password"
            type="password"
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500"
            placeholder="••••••••"
          />
        </div>
        
        <div v-if="error" class="text-sm text-red-600">
          {{ error }}
        </div>
        
        <button
          type="submit"
          :disabled="loading"
          class="w-full px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          {{ loading ? 'Logging in...' : 'Login' }}
        </button>
      </form>
      
      <p class="mt-4 text-sm text-center text-gray-600">
        Don't have an account?
        <NuxtLink to="/register" class="text-primary-600 hover:text-primary-700 font-medium">
          Register
        </NuxtLink>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
const { login } = useAuth()
const router = useRouter()

const email = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

const handleLogin = async () => {
  error.value = ''
  loading.value = true
  
  try {
    await login(email.value, password.value)
    await new Promise(resolve => setTimeout(resolve, 100))
    router.push('/modules')
  } catch (e: any) {
    error.value = e.message || 'Login failed'
  } finally {
    loading.value = false
  }
}
</script>
