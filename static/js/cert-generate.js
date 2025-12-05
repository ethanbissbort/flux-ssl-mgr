// Certificate Generation Page JavaScript

document.addEventListener('DOMContentLoaded', function() {
    const form = document.getElementById('cert-generate-form');
    const submitBtn = document.getElementById('submit-btn');
    const resultContainer = document.getElementById('result-container');
    const errorContainer = document.getElementById('error-container');
    const passwordProtectCheckbox = document.getElementById('password-protect');
    const passwordGroup = document.getElementById('password-group');
    const passwordInput = document.getElementById('key-password');
    const sanInput = document.getElementById('san-input');
    const addSanBtn = document.getElementById('add-san');
    const sanList = document.getElementById('san-list');
    const sansHidden = document.getElementById('sans');

    let sans = [];

    // Password protection toggle
    passwordProtectCheckbox.addEventListener('change', function() {
        if (this.checked) {
            passwordGroup.style.display = 'block';
            passwordInput.required = true;
        } else {
            passwordGroup.style.display = 'none';
            passwordInput.required = false;
            passwordInput.value = '';
        }
    });

    // SAN management
    function addSAN() {
        const value = sanInput.value.trim();
        if (!value) return;

        // Basic validation
        if (!value.match(/^(DNS|IP|EMAIL):.+/i)) {
            alert('SAN must be in format: DNS:hostname, IP:address, or EMAIL:address');
            return;
        }

        if (sans.includes(value)) {
            alert('This SAN is already added');
            return;
        }

        sans.push(value);
        updateSANList();
        sanInput.value = '';
        sansHidden.value = JSON.stringify(sans);
    }

    addSanBtn.addEventListener('click', addSAN);

    sanInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter') {
            e.preventDefault();
            addSAN();
        }
    });

    function updateSANList() {
        sanList.innerHTML = '';
        sans.forEach((san, index) => {
            const li = document.createElement('li');
            li.innerHTML = `
                ${san}
                <button type="button" class="remove-tag" data-index="${index}">×</button>
            `;
            sanList.appendChild(li);
        });

        // Add event listeners to remove buttons
        sanList.querySelectorAll('.remove-tag').forEach(btn => {
            btn.addEventListener('click', function() {
                const index = parseInt(this.dataset.index);
                sans.splice(index, 1);
                updateSANList();
                sansHidden.value = JSON.stringify(sans);
            });
        });
    }

    // Form submission
    form.addEventListener('submit', async function(e) {
        e.preventDefault();

        const formData = {
            common_name: document.getElementById('common-name').value.trim(),
            sans: sans,
            validity_days: parseInt(document.getElementById('validity-days').value),
            key_size: parseInt(document.getElementById('key-size').value),
            password_protect: passwordProtectCheckbox.checked
        };

        if (formData.password_protect) {
            formData.key_password = passwordInput.value;
        }

        // Validate
        if (!formData.common_name) {
            alert('Common Name is required');
            return;
        }

        // Show loading state
        const btnText = submitBtn.querySelector('.btn-text');
        const spinner = submitBtn.querySelector('.spinner');
        btnText.style.display = 'none';
        spinner.style.display = 'inline-block';
        submitBtn.disabled = true;

        try {
            const response = await fetch('/api/cert/generate', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(formData)
            });

            const data = await response.json();

            if (response.ok && data.success) {
                showResult(data.certificate);
            } else {
                showError(data.error || 'Failed to generate certificate');
            }
        } catch (error) {
            showError('Network error: ' + error.message);
        } finally {
            btnText.style.display = 'inline';
            spinner.style.display = 'none';
            submitBtn.disabled = false;
        }
    });

    function showResult(certificate) {
        form.style.display = 'none';
        errorContainer.style.display = 'none';
        resultContainer.style.display = 'block';

        document.getElementById('cert-subject').textContent = certificate.subject;
        document.getElementById('cert-serial').textContent = certificate.serial;
        document.getElementById('cert-not-before').textContent = new Date(certificate.not_before).toLocaleString();
        document.getElementById('cert-not-after').textContent = new Date(certificate.not_after).toLocaleString();
        document.getElementById('cert-sans').textContent = certificate.sans.length > 0
            ? certificate.sans.join(', ')
            : 'None';

        document.getElementById('cert-pem').value = certificate.pem;
        document.getElementById('key-pem').value = certificate.private_key;

        if (certificate.ca_chain) {
            document.getElementById('chain-pem').value = certificate.ca_chain;
            document.getElementById('chain-section').style.display = 'block';
            document.getElementById('download-chain').style.display = 'inline-block';
        }
    }

    function showError(message) {
        form.style.display = 'none';
        resultContainer.style.display = 'none';
        errorContainer.style.display = 'block';
        document.getElementById('error-message').textContent = message;
    }

    // Download buttons
    document.getElementById('download-cert').addEventListener('click', function() {
        downloadFile(document.getElementById('cert-pem').value, 'certificate.pem');
    });

    document.getElementById('download-key').addEventListener('click', function() {
        if (!confirm('⚠️ WARNING: This will download your private key. Store it securely!\n\nDo you want to continue?')) {
            return;
        }
        downloadFile(document.getElementById('key-pem').value, 'private-key.pem');
    });

    document.getElementById('download-chain').addEventListener('click', function() {
        downloadFile(document.getElementById('chain-pem').value, 'ca-chain.pem');
    });

    function downloadFile(content, filename) {
        const blob = new Blob([content], { type: 'application/x-pem-file' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }

    // Copy buttons
    document.getElementById('copy-cert').addEventListener('click', async function() {
        await copyToClipboard(this, document.getElementById('cert-pem').value);
    });

    document.getElementById('copy-key').addEventListener('click', async function() {
        await copyToClipboard(this, document.getElementById('key-pem').value);
    });

    document.getElementById('copy-chain').addEventListener('click', async function() {
        await copyToClipboard(this, document.getElementById('chain-pem').value);
    });

    async function copyToClipboard(button, text) {
        try {
            await navigator.clipboard.writeText(text);
            const originalText = button.textContent;
            button.textContent = 'Copied!';
            setTimeout(() => {
                button.textContent = originalText;
            }, 2000);
        } catch (err) {
            alert('Failed to copy to clipboard');
        }
    }

    // Generate another
    document.getElementById('generate-another').addEventListener('click', function() {
        form.style.display = 'block';
        resultContainer.style.display = 'none';
        errorContainer.style.display = 'none';
        form.reset();
        sans = [];
        updateSANList();
        sansHidden.value = '[]';
        passwordGroup.style.display = 'none';
        document.getElementById('chain-section').style.display = 'none';
    });

    // Try again
    document.getElementById('try-again').addEventListener('click', function() {
        form.style.display = 'block';
        errorContainer.style.display = 'none';
    });
});
