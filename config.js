// API Configuration
// Automatically detects environment and sets appropriate API base URL

const CONFIG = {
    // Detect environment
    isDevelopment: window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1',

    // API endpoints
    API_BASE_URL: (function() {
        // Check for explicit override in localStorage
        const override = localStorage.getItem('API_BASE_URL_OVERRIDE');
        if (override) return override;

        // Auto-detect based on hostname
        if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
            return 'http://localhost:8888';
        }

        // Production: Use Railway or Fly.io deployment
        // Update this URL after deploying backend
        return 'https://veto-frontier-backend.up.railway.app'; // Replace with actual URL
    })(),

    // Default org ID
    DEFAULT_ORG_ID: '00000000-0000-0000-0000-000000000001',

    // Demo mode (for UI testing without backend)
    DEMO_MODE: false, // Set to true to use mock API responses

    // Debug logging
    DEBUG: true
};

// Helper function for API calls
async function apiCall(endpoint, options = {}) {
    const url = `${CONFIG.API_BASE_URL}${endpoint}`;

    if (CONFIG.DEBUG) {
        console.log(`[API] ${options.method || 'GET'} ${url}`, options.body || '');
    }

    try {
        const response = await fetch(url, {
            ...options,
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            }
        });

        const data = await response.json();

        if (CONFIG.DEBUG) {
            console.log(`[API] Response ${response.status}:`, data);
        }

        if (!response.ok) {
            throw new Error(data.error || `API error: ${response.status}`);
        }

        return { success: true, data, status: response.status };
    } catch (error) {
        console.error('[API] Error:', error);

        if (CONFIG.DEMO_MODE) {
            console.warn('[API] Demo mode active, returning mock data');
            return { success: false, error: error.message, demo: true };
        }

        return { success: false, error: error.message };
    }
}

// Export for use in index.html
window.CONFIG = CONFIG;
window.apiCall = apiCall;

console.log('[CONFIG] API Base URL:', CONFIG.API_BASE_URL);
console.log('[CONFIG] Environment:', CONFIG.isDevelopment ? 'Development' : 'Production');
console.log('[CONFIG] Demo Mode:', CONFIG.DEMO_MODE ? 'ON' : 'OFF');
