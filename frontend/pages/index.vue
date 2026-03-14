<template>
  <div class="px-4 py-12 sm:px-6 lg:px-8">
    <div class="text-center">
      <h1 class="text-4xl font-extrabold text-gray-900 sm:text-5xl sm:tracking-tight lg:text-6xl">
        Welcome to DRASM
      </h1>
      <p class="mt-5 max-w-xl mx-auto text-xl text-gray-500">
        Distributed Runtime for Asynchronous Serverless Modules
      </p>
      <div v-if="!isAuthenticated" class="mt-8 flex justify-center gap-4">
        <NuxtLink
          to="/register"
          class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
        >
          Get Started
        </NuxtLink>
        <NuxtLink
          to="/login"
          class="inline-flex items-center px-6 py-3 border border-gray-300 text-base font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
        >
          Sign In
        </NuxtLink>
      </div>

      <div v-else class="mt-8 flex justify-center gap-4">
        <NuxtLink
          to="/modules"
          class="inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
        >
          Browse Modules
        </NuxtLink>
        <NuxtLink
          to="/jobs/create"
          class="inline-flex items-center px-6 py-3 border border-gray-300 text-base font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
        >
          Create Job
        </NuxtLink>
      </div>
    </div>

    <div class="mt-20">
      <div class="grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-3">
        <div class="pt-6">
          <div class="flow-root bg-white rounded-lg px-6 pb-8">
            <div class="-mt-6">
              <div class="inline-flex items-center justify-center p-3 bg-indigo-500 rounded-md shadow-lg">
                <svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <h3 class="mt-8 text-lg font-medium text-gray-900 tracking-tight">Fast & Efficient</h3>
              <p class="mt-5 text-base text-gray-500">
                Built with Rust for maximum performance and reliability
              </p>
            </div>
          </div>
        </div>

        <div class="pt-6">
          <div class="flow-root bg-white rounded-lg px-6 pb-8">
            <div class="-mt-6">
              <div class="inline-flex items-center justify-center p-3 bg-indigo-500 rounded-md shadow-lg">
                <svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
              </div>
              <h3 class="mt-8 text-lg font-medium text-gray-900 tracking-tight">WebAssembly Modules</h3>
              <p class="mt-5 text-base text-gray-500">
                Run WebAssembly modules in a secure, sandboxed environment
              </p>
            </div>
          </div>
        </div>

        <div class="pt-6">
          <div class="flow-root bg-white rounded-lg px-6 pb-8">
            <div class="-mt-6">
              <div class="inline-flex items-center justify-center p-3 bg-indigo-500 rounded-md shadow-lg">
                <svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
                </svg>
              </div>
              <h3 class="mt-8 text-lg font-medium text-gray-900 tracking-tight">Distributed Processing</h3>
              <p class="mt-5 text-base text-gray-500">
                Scale your workloads across multiple workers with Apache Kafka
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const { isAuthenticated } = useAuth();
</script>
