<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Box, User, LogIn, UserPlus, PackageOpen, Play, MoreVertical, RefreshCw } from 'lucide-vue-next'

const instances = ref([])
const accounts = ref([])
const isLoadingInstances = ref(true)
const instancesError = ref('')

const msaPromptVisible = ref(false)
const msaUri = ref('')
const msaCode = ref('----')

const offlineName = ref('')
const isAddingOffline = ref(false)

async function loadInstances() {
  isLoadingInstances.value = true
  instancesError.value = ''
  instances.value = []

  try {
    const res = await invoke("list_instances")
    instances.value = res || []
  } catch (err) {
    instancesError.value = `Error Loading Instances: ${err}`
  } finally {
    isLoadingInstances.value = false
  }
}

async function loadAccounts() {
  try {
    const res = await invoke("list_accounts")
    accounts.value = res || []
  } catch (err) {
    console.error("Failed to load accounts", err)
  }
}

async function addOfflineAccount() {
  const name = offlineName.value.trim()
  if (!name) return
  isAddingOffline.value = true
  try {
    await invoke("add_offline_account", { username: name })
    offlineName.value = ''
    await loadAccounts()
  } catch (e) {
    alert("Error adding offline account: " + e)
  } finally {
    isAddingOffline.value = false
  }
}

async function addMsaAccount() {
  msaPromptVisible.value = true
  msaUri.value = 'Loading...'
  msaCode.value = '----'
  
  try {
    await invoke("login_msa")
  } catch (e) {
    alert("Error adding MSA account: " + e)
  } finally {
    msaPromptVisible.value = false
    await loadAccounts()
  }
}

const netProgressVisible = ref(false)
const netJobName = ref('')
const netJobCompleted = ref(0)
const netJobTotal = ref(0)

async function launchInstance(id) {
  try {
    netProgressVisible.value = true;
    netJobName.value = 'Preparing Launch...';
    netJobCompleted.value = 0;
    netJobTotal.value = 1;
    await invoke("launch_instance", { id })
  } catch (err) {
    alert(`Launch failed: ${err}`)
  } finally {
    setTimeout(() => { netProgressVisible.value = false; }, 2000);
  }
}

function getAccountName(acc) {
  if (acc.type === "MSA" && acc.ygg?.extra?.userName) return acc.ygg.extra.userName;
  if (acc.profile?.name && acc.profile.name !== "") return acc.profile.name;
  return "Unknown";
}

onMounted(() => {
  loadInstances()
  loadAccounts()

  listen("device-code", (event) => {
    const data = event.payload
    msaUri.value = data.uri
    msaCode.value = data.code
  })

  listen("net-progress", (event) => {
    const data = event.payload;
    netProgressVisible.value = true;
    netJobName.value = data.job_name;
    netJobCompleted.value = data.completed_actions;
    netJobTotal.value = data.total_actions;
  })
})
</script>

<template>
  <div id="app-container">
    <header>
      <h1>
        <Box />
        KCraft
      </h1>
      <button class="btn-secondary" @click="loadInstances">
        <RefreshCw :size="16" />
        Refresh
      </button>
    </header>

    <div class="layout">
      <aside>
        <div class="section-header">Active Accounts</div>
        <div class="accounts-list">
          <p v-if="accounts.length === 0" style="text-align:center;color:var(--text-secondary);font-size:0.875rem;padding:1rem;">
            No active accounts.
          </p>
          <div v-for="acc in accounts" :key="acc.id || getAccountName(acc)" class="card account-card">
            <div class="account-avatar">
              <User :size="20" style="color: var(--text-secondary)" />
            </div>
            <div class="account-info">
              <div class="account-name">{{ getAccountName(acc) }}</div>
              <div class="account-type">
                <div :style="{ width: '6px', height: '6px', borderRadius: '50%', background: acc.type === 'MSA' ? '#10b981' : '#6366f1' }"></div>
                {{ acc.type === "MSA" ? "Microsoft Account" : "Offline Account" }}
              </div>
            </div>
          </div>
        </div>
        
        <div style="position: relative;">
          <div v-if="msaPromptVisible" id="msa-prompt">
            <p style="color: var(--text-secondary); margin-bottom: 0.5rem;">Microsoft Authentication</p>
            <p>1. Open <a :href="msaUri" target="_blank" style="color:#818cf8; font-weight:600; text-decoration: none;">{{ msaUri }}</a></p>
            <p>2. Enter this code:</p>
            <div class="code-display">{{ msaCode }}</div>
            <p style="font-size: 0.75rem; color: var(--text-secondary);">Waiting for approval...</p>
          </div>

          <div class="controls">
            <button class="btn-secondary btn-full" @click="addMsaAccount" style="border-color: #00a4ef;">
              <LogIn :size="16" /> Add Microsoft Account
            </button>
            <div class="input-group">
              <input type="text" v-model="offlineName" placeholder="Offline Username" @keyup.enter="addOfflineAccount" />
              <button class="btn-secondary" @click="addOfflineAccount" :disabled="isAddingOffline">
                <div v-if="isAddingOffline" class="spinner" style="width:16px;height:16px;border-width:2px;"></div>
                <UserPlus v-else :size="16" />
              </button>
            </div>
          </div>
        </div>
      </aside>

      <main>
        <div v-if="netProgressVisible" class="progress-banner">
          <div class="progress-info">
            <span style="font-weight: 600; color: #f8fafc;">{{ netJobName }}</span>
            <span style="color: #94a3b8; font-size: 0.875rem;">{{ netJobCompleted }} / {{ netJobTotal }} completed</span>
          </div>
          <div class="progress-track">
            <div class="progress-fill" :style="{ width: netJobTotal > 0 ? `${(netJobCompleted / netJobTotal) * 100}%` : '0%' }"></div>
          </div>
        </div>

        <div v-if="isLoadingInstances" id="loading">
          <div class="spinner"></div>
          <span>Loading instances...</span>
        </div>
        
        <div v-else-if="instancesError" id="error" style="padding: 2rem; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 12px; color: #fca5a5;">
          <strong>Error Loading Instances:</strong><br/>{{ instancesError }}
        </div>
        
        <div v-else-if="instances.length === 0" style="grid-column: 1/-1; text-align:center; padding: 4rem; color: var(--text-secondary);">
          <PackageOpen :size="48" style="margin-bottom: 1rem; opacity: 0.5;" /><br/>
          No instances found.
        </div>
        
        <div v-else class="instances-grid">
          <div v-for="inst in instances" :key="inst.id" class="card instance-card">
            <div class="instance-header">
              <div class="instance-icon">
                <Box />
              </div>
              <div>
                <div class="instance-title">{{ inst.name }}</div>
                <div class="instance-meta">{{ inst.id }}</div>
              </div>
            </div>
            
            <p v-if="inst.notes" style="font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 1rem; flex: 1;">
              {{ inst.notes }}
            </p>
            <div v-else style="flex: 1;"></div>
            
            <div class="instance-footer">
              <button class="play-btn" @click.stop="launchInstance(inst.id)">
                <Play :size="16" /> Play
              </button>
              <button class="btn-secondary" style="padding: 0.5rem; background: transparent; border: none;">
                <MoreVertical :size="16" />
              </button>
            </div>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<style>
:root {
  --bg-base: #0f172a;
  --bg-surface: #1e293b;
  --bg-surface-hover: #334155;
  --border-color: #334155;
  --text-primary: #f8fafc;
  --text-secondary: #94a3b8;
  --accent: #6366f1;
  --accent-hover: #4f46e5;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: var(--bg-base);
  color: var(--text-primary);
  overflow: hidden;
  height: 100vh;
}

#app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

/* Glass Header */
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 2rem;
  background: rgba(30, 41, 59, 0.7);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid var(--border-color);
  z-index: 10;
}

header h1 {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: -0.025em;
  background: linear-gradient(to right, #818cf8, #c084fc);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

/* Layout */
.layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Sidebar */
aside {
  width: 320px;
  background: rgba(15, 23, 42, 0.5);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  backdrop-filter: blur(8px);
}

.section-header {
  padding: 1.25rem 1.5rem 0.5rem;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
  font-weight: 600;
}

.accounts-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

/* Cards */
.card {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 1rem;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.card::before {
  content: '';
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background: linear-gradient(120deg, rgba(255,255,255,0) 30%, rgba(255,255,255,0.05) 50%, rgba(255,255,255,0) 70%);
  transform: translateX(-100%);
  transition: transform 0.6s;
}

.card:hover {
  background: var(--bg-surface-hover);
  transform: translateY(-2px);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3);
  border-color: var(--accent);
}

.card:hover::before {
  transform: translateX(100%);
}

.account-card {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.account-avatar {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
}

.account-info {
  flex: 1;
}

.account-name {
  font-weight: 600;
  font-size: 0.875rem;
}

.account-type {
  font-size: 0.75rem;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin-top: 0.125rem;
}

/* Main Content */
main {
  flex: 1;
  overflow-y: auto;
  padding: 2rem;
  position: relative;
}

.instances-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1.5rem;
}

.instance-card {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.instance-header {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  margin-bottom: 1rem;
}

.instance-icon {
  width: 56px;
  height: 56px;
  border-radius: 12px;
  background: var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
}

.instance-title {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

.instance-meta {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.progress-banner {
  background: var(--bg-surface);
  border: 1px solid var(--accent);
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1.5rem;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  animation: slideDown 0.3s ease-out;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.progress-track {
  width: 100%;
  height: 8px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(to right, #6366f1, #c084fc);
  transition: width 0.3s ease-out;
}

.instance-footer {
  margin-top: auto;
  padding-top: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--border-color);
}

.play-btn {
  background: var(--accent);
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.play-btn:hover {
  background: var(--accent-hover);
  transform: scale(1.05);
}

/* Controls & Inputs */
.controls {
  padding: 1.5rem;
  background: rgba(15, 23, 42, 0.8);
  border-top: 1px solid var(--border-color);
}

.input-group {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

input[type="text"] {
  flex: 1;
  padding: 0.625rem 1rem;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: rgba(0, 0, 0, 0.2);
  color: var(--text-primary);
  font-family: inherit;
  transition: all 0.2s;
}

input[type="text"]:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

.btn-secondary {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 0.625rem 1rem;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.btn-secondary:hover {
  background: var(--bg-surface-hover);
}

.btn-full {
  width: 100%;
}

/* Modal / Prompts */
#msa-prompt {
  position: absolute;
  bottom: 100%;
  left: 0; right: 0;
  background: rgba(30, 41, 59, 0.95);
  backdrop-filter: blur(12px);
  padding: 1.5rem;
  border-top: 1px solid var(--accent);
  border-bottom: 1px solid var(--accent);
  text-align: center;
  z-index: 20;
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.code-display {
  font-size: 2rem;
  font-weight: 800;
  letter-spacing: 4px;
  color: #818cf8;
  margin: 1rem 0;
  user-select: all;
}

/* Scrollbars */
::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border-color); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: var(--text-secondary); }

#loading { text-align: center; padding: 4rem; color: var(--text-secondary); display: flex; flex-direction: column; align-items: center; gap: 1rem; }
.spinner { border: 3px solid rgba(255,255,255,0.1); border-left-color: var(--accent); border-radius: 50%; width: 24px; height: 24px; animation: spin 1s linear infinite; }
@keyframes spin { 100% { transform: rotate(360deg); } }
</style>
