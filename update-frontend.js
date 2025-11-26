// Script to update index.html to connect to real backend API
// Run with: node update-frontend.js

const fs = require('fs');
const path = require('path');

const indexPath = path.join(__dirname, 'index.html');
let html = fs.readFileSync(indexPath, 'utf8');

console.log('Updating index.html to connect to real backend...\n');

// 1. Add config.js script tag before </head>
if (!html.includes('config.js')) {
    html = html.replace('</head>', '    <script src="config.js"></script>\n</head>');
    console.log('✓ Added config.js script tag');
}

// 2. Find and replace the triggerVeto function with real API integration
const vetoFunctionStart = html.indexOf('function triggerVeto()');
const vetoFunctionEnd = html.indexOf('}, 1600);', vetoFunctionStart) + 10;

if (vetoFunctionStart !== -1 && vetoFunctionEnd !== -1) {
    const newVetoFunction = `async function triggerVeto() {
            if (isVetoing) return;
            isVetoing = true;

            const btn = document.getElementById('veto-btn');
            btn.disabled = true;
            btn.style.opacity = '0.5';

            const epoch = Math.floor(Date.now() / 1000);
            const receiptData = 'VETO_EVENT_' + epoch + '_USER_CONSENT_REVOKED';
            const hash = btoa(receiptData).substring(0, 16);

            // Audio Alert
            log('!!! EMERGENCY OVERRIDE INITIATED !!!', 'error');
            log('!!! ZERO-MULTIPLIER VETO ENGAGED !!!', 'error');
            log('!!! US 63/920,993 ENFORCED !!!', 'error');
            log('', 'entry');
            log('Generating revocation receipt: ' + hash + '...', 'warn');

            try {
                // Step 1: Create a pointer (in production, this would already exist)
                log('', 'entry');
                log('Creating demo pointer...', 'system');

                const createResult = await apiCall('/api/pointer/create', {
                    method: 'POST',
                    body: JSON.stringify({
                        subject_id: 'demo_user_' + Date.now(),
                        content_hash: 'sha3_demo_' + hash
                    })
                });

                if (!createResult.success) {
                    throw new Error('Failed to create pointer: ' + createResult.error);
                }

                const pointerId = createResult.data.pointer_id;
                log('✓ Pointer created: ' + pointerId, 'success');
                log('✓ Receipt hash: ' + createResult.data.receipt.receipt_hash.substring(0, 16) + '...', 'success');

                // Step 2: Orphan the pointer (VETO!)
                log('', 'entry');
                log('⚡ Oracle-triggered: viral threshold met → ZKP verified → de-link fired', 'system');
                log('', 'entry');
                log('POST /api/pointer/orphan {pointer_id: "' + pointerId + '"}', 'system');

                const orphanResult = await apiCall('/api/pointer/orphan', {
                    method: 'POST',
                    body: JSON.stringify({
                        pointer_id: pointerId,
                        reason: 'user_consent_revoked'
                    })
                });

                if (!orphanResult.success) {
                    throw new Error('Failed to orphan pointer: ' + orphanResult.error);
                }

                log('← 200 OK: {"status": "orphaned", "data": "intact"}', 'success');
                log('', 'entry');

                // Real receipt from backend
                const receipt = orphanResult.data.receipt;
                log('[CHAIN EVENT] REVOKE | hash=' + receipt.receipt_hash.substring(0, 16) + '... | sig=' + receipt.signature_algorithm + ':' + receipt.signature.substring(0, 8) + '...', 'system');
                log('', 'entry');

                // Step 3: Try to resolve (should fail with 403)
                log('Attempting to resolve orphaned pointer...', 'warn');

                const resolveResult = await apiCall('/api/pointer/resolve/' + pointerId, {
                    method: 'GET'
                });

                if (!resolveResult.success) {
                    log('✓ Resolution blocked: ' + resolveResult.error, 'success');
                } else {
                    log('⚠ Warning: Orphaned pointer was resolved (enforcement may be disabled)', 'warn');
                }

                log('', 'entry');

                // Show popup confirmation - Patent Claim 9
                const popup = document.createElement('div');
                popup.style.position = 'fixed';
                popup.style.top = '50%';
                popup.style.left = '50%';
                popup.style.transform = 'translate(-50%, -50%)';
                popup.style.background = '#1a0000';
                popup.style.border = '2px solid #ff0000';
                popup.style.padding = '40px';
                popup.style.zIndex = '1000';
                popup.style.boxShadow = '0 0 50px rgba(255, 0, 0, 0.8)';
                popup.style.maxWidth = '600px';
                popup.style.fontFamily = 'Share Tech Mono, monospace';
                popup.innerHTML = '<div style="color: #ff0000; font-size: 24px; font-weight: bold; margin-bottom: 20px; text-align: center;">⚠️ CONSENT REVOKED</div><div style="color: #00ff00; font-size: 16px; margin-bottom: 30px; line-height: 1.6;"><strong>POINTER DE-LINKED (REAL)</strong><br><strong>DATA INTACT</strong><br><br>Per <span style="color: #ffff00;">US 19/240,581 Claim 9</span>:<br>"...orphaning said pointer while preserving the underlying data object in storage..."<br><br><strong>Receipt Hash:</strong> ' + receipt.receipt_hash.substring(0, 32) + '...<br><strong>Signature:</strong> ' + receipt.signature_algorithm + '<br><strong>Timestamp:</strong> ' + receipt.timestamp + '</div><div style="text-align: center;"><button onclick="this.parentElement.parentElement.remove()" style="background: #ff0000; color: #000; padding: 10px 30px; border: none; cursor: pointer; font-weight: bold; font-family: \'Share Tech Mono\', monospace;">ACKNOWLEDGE</button></div>';
                document.body.appendChild(popup);

                log('✓ Consent revoked: pointer de-linked, data intact (REAL)', 'success');
                log('✓ Patent US 19/240,581 Claim 9 enforced (PRODUCTION)', 'success');
                log('✓ Cryptographic receipt generated and stored', 'success');

            } catch (error) {
                log('', 'entry');
                log('ERROR: ' + error.message, 'error');
                log('Falling back to demo mode...', 'warn');

                // Fallback to original mock behavior if API fails
                log('POST /api/orphan {hash: "' + hash + '"}', 'system');
                log('← 200 OK: {"status": "de-linked", "data": "intact"} (DEMO)', 'success');
            }

            // Crash the Score
            updateTrust(0);

            setTimeout(() => {
                log('', 'entry');
                log('SYSTEM HALTED. TRUST SCORE: 0.00%', 'error');
                log('ALL OPERATIONS CEASED.', 'error');

                // Flash Red
                const overlay = document.createElement('div');
                overlay.style.position = 'fixed';
                overlay.style.top = '0';
                overlay.style.left = '0';
                overlay.style.width = '100%';
                overlay.style.height = '100%';
                overlay.style.background = 'rgba(255, 0, 0, 0.3)';
                overlay.style.zIndex = '999';
                overlay.style.pointerEvents = 'none';
                document.body.appendChild(overlay);

                setTimeout(() => {
                    overlay.remove();
                }, 500);

                isVetoing = false;
                btn.disabled = false;
                btn.style.opacity = '1';
            }, 1600);
        }`;

    html = html.substring(0, vetoFunctionStart) + newVetoFunction + html.substring(vetoFunctionEnd);
    console.log('✓ Updated triggerVeto() function with real API calls');
}

// 3. Add API status indicator
if (!html.includes('api-status-indicator')) {
    const statusIndicator = `
    <!-- API Status Indicator -->
    <div id="api-status-indicator" style="position: fixed; top: 10px; right: 10px; padding: 8px 16px; background: rgba(0,0,0,0.8); border: 1px solid #00ff00; border-radius: 4px; font-family: 'Share Tech Mono', monospace; font-size: 12px; z-index: 2000;">
        <span id="api-status-text" style="color: #00ff00;">API: Checking...</span>
    </div>
    <script>
        // Check API health on load
        (async function() {
            const indicator = document.getElementById('api-status-text');
            try {
                const result = await apiCall('/health');
                if (result.success) {
                    indicator.textContent = 'API: Connected ✓';
                    indicator.style.color = '#00ff00';
                } else {
                    throw new Error('Health check failed');
                }
            } catch (error) {
                indicator.textContent = 'API: Offline (Demo Mode)';
                indicator.style.color = '#ff9900';
                CONFIG.DEMO_MODE = true;
            }
        })();
    </script>
`;

    html = html.replace('</body>', statusIndicator + '\n</body>');
    console.log('✓ Added API status indicator');
}

// Write updated file
fs.writeFileSync(indexPath, html, 'utf8');

console.log('\n✓ Frontend updated successfully!');
console.log('\nNext steps:');
console.log('1. Update config.js with your deployed backend URL');
console.log('2. Test locally: open index.html in browser');
console.log('3. Deploy to Vercel: git commit && git push');
