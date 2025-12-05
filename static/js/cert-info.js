// Certificate Info Page JavaScript

document.addEventListener('DOMContentLoaded', function() {
    const form = document.getElementById('cert-info-form');
    const fileInput = document.getElementById('cert-file');
    const dropZone = document.getElementById('cert-drop-zone');
    const uploadPlaceholder = dropZone.querySelector('.upload-placeholder');
    const fileInfo = document.getElementById('file-info');
    const removeFileBtn = document.getElementById('remove-file');
    const submitBtn = document.getElementById('submit-btn');
    const resultContainer = document.getElementById('result-container');
    const errorContainer = document.getElementById('error-container');

    let selectedFile = null;

    // File drag and drop
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
        dropZone.addEventListener(eventName, preventDefaults, false);
    });

    function preventDefaults(e) {
        e.preventDefault();
        e.stopPropagation();
    }

    ['dragenter', 'dragover'].forEach(eventName => {
        dropZone.addEventListener(eventName, () => {
            dropZone.classList.add('drag-over');
        });
    });

    ['dragleave', 'drop'].forEach(eventName => {
        dropZone.addEventListener(eventName, () => {
            dropZone.classList.remove('drag-over');
        });
    });

    dropZone.addEventListener('drop', handleDrop);

    function handleDrop(e) {
        const dt = e.dataTransfer;
        const files = dt.files;

        if (files.length > 0) {
            fileInput.files = files;
            handleFileSelect({ target: fileInput });
        }
    }

    // File selection
    fileInput.addEventListener('change', handleFileSelect);

    function handleFileSelect(e) {
        const file = e.target.files[0];
        if (file) {
            selectedFile = file;
            displayFileInfo(file);
        }
    }

    function displayFileInfo(file) {
        uploadPlaceholder.style.display = 'none';
        fileInfo.style.display = 'block';
        fileInfo.querySelector('.file-name').textContent = file.name;
        fileInfo.querySelector('.file-size').textContent = formatFileSize(file.size);
    }

    function formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }

    // Remove file
    removeFileBtn.addEventListener('click', function() {
        fileInput.value = '';
        selectedFile = null;
        uploadPlaceholder.style.display = 'block';
        fileInfo.style.display = 'none';
    });

    // Form submission
    form.addEventListener('submit', async function(e) {
        e.preventDefault();

        if (!selectedFile) {
            showError('Please select a certificate file');
            return;
        }

        // Show loading state
        const btnText = submitBtn.querySelector('.btn-text');
        const spinner = submitBtn.querySelector('.spinner');
        btnText.style.display = 'none';
        spinner.style.display = 'inline-block';
        submitBtn.disabled = true;

        const formData = new FormData();
        formData.append('cert_file', selectedFile);

        try {
            const response = await fetch('/api/cert/info', {
                method: 'POST',
                body: formData
            });

            const data = await response.json();

            if (response.ok && data.success) {
                showResult(data.certificate);
            } else {
                showError(data.error || 'Failed to parse certificate');
            }
        } catch (error) {
            showError('Network error: ' + error.message);
        } finally {
            btnText.style.display = 'inline';
            spinner.style.display = 'none';
            submitBtn.disabled = false;
        }
    });

    function showResult(cert) {
        form.style.display = 'none';
        errorContainer.style.display = 'none';
        resultContainer.style.display = 'block';

        // Show validity status
        const statusDiv = document.getElementById('cert-status');
        if (cert.validity.is_expired) {
            statusDiv.className = 'cert-status expired';
            statusDiv.textContent = '❌ Certificate is EXPIRED';
        } else if (cert.validity.is_expiring_soon) {
            statusDiv.className = 'cert-status expiring-soon';
            statusDiv.textContent = `⚠️ Certificate expires soon (${cert.validity.days_remaining} days remaining)`;
        } else {
            statusDiv.className = 'cert-status valid';
            statusDiv.textContent = `✓ Certificate is valid (${cert.validity.days_remaining} days remaining)`;
        }

        // Basic information
        document.getElementById('cert-version').textContent = 'v' + cert.version;
        document.getElementById('cert-serial').textContent = cert.serial_number;
        document.getElementById('cert-sig-alg').textContent = cert.signature_algorithm;

        // Issuer
        const issuerDl = document.getElementById('cert-issuer');
        issuerDl.innerHTML = '';
        Object.entries(cert.issuer).forEach(([key, value]) => {
            const dt = document.createElement('dt');
            dt.textContent = key + ':';
            const dd = document.createElement('dd');
            dd.textContent = value;
            issuerDl.appendChild(dt);
            issuerDl.appendChild(dd);
        });

        // Subject
        const subjectDl = document.getElementById('cert-subject');
        subjectDl.innerHTML = '';
        Object.entries(cert.subject).forEach(([key, value]) => {
            const dt = document.createElement('dt');
            dt.textContent = key + ':';
            const dd = document.createElement('dd');
            dd.textContent = value;
            subjectDl.appendChild(dt);
            subjectDl.appendChild(dd);
        });

        // Validity
        document.getElementById('cert-not-before').textContent = new Date(cert.validity.not_before).toLocaleString();
        document.getElementById('cert-not-after').textContent = new Date(cert.validity.not_after).toLocaleString();
        document.getElementById('cert-days-remaining').textContent = cert.validity.days_remaining + ' days';

        const validityStatus = document.getElementById('cert-validity-status');
        if (cert.validity.is_expired) {
            validityStatus.innerHTML = '<span style="color: var(--danger-color)">Expired</span>';
        } else if (cert.validity.is_expiring_soon) {
            validityStatus.innerHTML = '<span style="color: var(--warning-color)">Expiring Soon</span>';
        } else {
            validityStatus.innerHTML = '<span style="color: var(--success-color)">Valid</span>';
        }

        // Public Key
        document.getElementById('key-algorithm').textContent = cert.public_key.algorithm;
        document.getElementById('key-size').textContent = cert.public_key.size + ' bits';
        document.getElementById('key-exponent').textContent = cert.public_key.exponent || 'N/A';

        // Fingerprints
        document.getElementById('fingerprint-sha1').textContent = cert.fingerprints.sha1;
        document.getElementById('fingerprint-sha256').textContent = cert.fingerprints.sha256;

        // Subject Alternative Names
        if (cert.subject_alternative_names && cert.subject_alternative_names.length > 0) {
            const sansSection = document.getElementById('sans-section');
            sansSection.style.display = 'block';
            const sansList = document.getElementById('cert-sans');
            sansList.innerHTML = '';
            cert.subject_alternative_names.forEach(san => {
                const li = document.createElement('li');
                li.textContent = san;
                sansList.appendChild(li);
            });
        }

        // Extensions
        if (cert.extensions && cert.extensions.length > 0) {
            const extensionsSection = document.getElementById('extensions-section');
            extensionsSection.style.display = 'block';
            const extensionsDiv = document.getElementById('cert-extensions');
            extensionsDiv.innerHTML = '';
            cert.extensions.forEach(ext => {
                const extDiv = document.createElement('div');
                extDiv.className = 'extension';
                extDiv.innerHTML = `
                    <div class="extension-name">${ext.name}${ext.critical ? ' (Critical)' : ''}</div>
                    <div class="extension-oid">OID: ${ext.oid}</div>
                    <div class="extension-value">${ext.value}</div>
                `;
                extensionsDiv.appendChild(extDiv);
            });
        }

        // PEM
        document.getElementById('cert-pem').value = cert.pem;
    }

    function showError(message) {
        form.style.display = 'none';
        resultContainer.style.display = 'none';
        errorContainer.style.display = 'block';
        document.getElementById('error-message').textContent = message;
    }

    // Copy PEM
    document.getElementById('copy-pem').addEventListener('click', async function() {
        const pem = document.getElementById('cert-pem').value;
        try {
            await navigator.clipboard.writeText(pem);
            this.textContent = 'Copied!';
            setTimeout(() => {
                this.textContent = 'Copy to Clipboard';
            }, 2000);
        } catch (err) {
            alert('Failed to copy to clipboard');
        }
    });

    // View another
    document.getElementById('view-another').addEventListener('click', function() {
        form.style.display = 'block';
        resultContainer.style.display = 'none';
        errorContainer.style.display = 'none';
        form.reset();
        removeFileBtn.click();
    });

    // Try again
    document.getElementById('try-again').addEventListener('click', function() {
        form.style.display = 'block';
        errorContainer.style.display = 'none';
    });
});
