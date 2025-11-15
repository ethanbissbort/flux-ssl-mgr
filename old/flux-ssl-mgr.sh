#!/bin/bash

# SSL Certificate Generation Script
# This script automates the process of generating SSL certificates using an intermediate CA
# 
# Usage:
#   Interactive mode: ./generate-ssl-cert.sh
#   Non-interactive mode: ./generate-ssl-cert.sh <hostname> <ip-address> [additional-sans...]
#   Help: ./generate-ssl-cert.sh -h|--help
#
# Examples:
#   ./generate-ssl-cert.sh web01 192.168.1.100
#   ./generate-ssl-cert.sh api-server 10.0.1.50 DNS:api.example.com DNS:api-alt.example.com
#   ./generate-ssl-cert.sh database 172.16.0.10 IP:172.16.0.11 DNS:db.local DNS:database.local

set -e  # Exit on any error

# Function to validate IP address format
validate_ip() {
    local ip=$1
    if [[ $ip =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
        IFS='.' read -ra ADDR <<< "$ip"
        for i in "${ADDR[@]}"; do
            if [[ $i -gt 255 ]]; then
                return 1
            fi
        done
        return 0
    else
        return 1
    fi
}

# Function to validate hostname format
validate_hostname() {
    local hostname=$1
    if [[ $hostname =~ ^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$ ]]; then
        return 0
    else
        return 1
    fi
}

# Function to validate SAN entry format
validate_san_entry() {
    local san_entry=$1
    if [[ $san_entry =~ ^(DNS|IP|EMAIL|URI):.+ ]]; then
        local type=$(echo "$san_entry" | cut -d: -f1)
        local value=$(echo "$san_entry" | cut -d: -f2-)
        
        case $type in
            DNS)
                # Basic DNS name validation
                if [[ $value =~ ^[a-zA-Z0-9]([a-zA-Z0-9\-\.]{0,253}[a-zA-Z0-9])?$ ]]; then
                    return 0
                fi
                ;;
            IP)
                # Use existing IP validation function
                if validate_ip "$value"; then
                    return 0
                fi
                ;;
            EMAIL)
                # Basic email validation
                if [[ $value =~ ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$ ]]; then
                    return 0
                fi
                ;;
            URI)
                # Basic URI validation (must start with http/https/ftp)
                if [[ $value =~ ^(https?|ftp)://.+ ]]; then
                    return 0
                fi
                ;;
        esac
    fi
    return 1
}

# Function to collect additional SAN entries interactively
collect_san_entries() {
    local additional_sans=()
    
    echo
    echo "=== Additional Subject Alternative Names (Optional) ==="
    echo "You can add additional DNS names, IP addresses, emails, or URIs to the certificate."
    echo "Press Enter without input to finish, or enter SAN entries in format: TYPE:value"
    echo "Examples: DNS:api.example.com, IP:10.0.1.1, EMAIL:admin@example.com, URI:https://api.local"
    echo
    
    while true; do
        read -p "Enter additional SAN entry (or press Enter to finish): " san_entry
        
        if [[ -z "$san_entry" ]]; then
            break
        fi
        
        if validate_san_entry "$san_entry"; then
            additional_sans+=("$san_entry")
            echo "Added: $san_entry"
        else
            echo "Error: Invalid SAN entry format. Use TYPE:value (e.g., DNS:example.com, IP:192.168.1.1)"
        fi
    done
    
    # Return the array as a space-separated string
    echo "${additional_sans[@]}"
}

# Function to show usage information
show_usage() {
    echo "SSL Certificate Generation Script"
    echo
    echo "Usage:"
    echo "  Interactive mode:     $0"
    echo "  Non-interactive mode: $0 <hostname> <ip-address> [additional-sans...]"
    echo "  Help:                 $0 -h|--help"
    echo
    echo "Arguments:"
    echo "  hostname           - The hostname identifier (will create <hostname>.fluxlab.systems)"
    echo "  ip-address         - The primary IP address for Subject Alternative Name"
    echo "  additional-sans    - Optional additional SAN entries (format: TYPE:value)"
    echo "  Remote execution:     curl -sSL https://raw.githubusercontent.com/ethanbissbort/flux-ssl-mgr/main/flux-ssl-mgr.sh | bash -s -- <hostname> <ip-address> [additional-sans...]"
    echo
    echo "SAN Types:"
    echo "  DNS:domain.com     - Additional DNS names"
    echo "  IP:192.168.1.1     - Additional IP addresses"
    echo "  EMAIL:user@dom.com - Email addresses"
    echo "  URI:https://...    - URIs"
    echo
    echo "Examples:"
    echo "  $0 web01 192.168.1.100"
    echo "  $0 api-server 10.0.1.50 DNS:api.example.com DNS:api-alt.example.com"
    echo "  $0 database 172.16.0.10 IP:172.16.0.11 DNS:db.local DNS:database.local"
    echo "  $0 mail 192.168.1.200 DNS:mail.example.com EMAIL:admin@example.com"
    echo
    exit 1
}

echo "=== SSL Certificate Generation Script ==="
echo

# Check for help flag
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_usage
fi

# Determine mode based on arguments
if [[ $# -eq 0 ]]; then
    # Interactive mode
    echo "Running in interactive mode..."
    echo
    
    # Prompt for hostname/identifier
    while true; do
        read -p "Enter the hostname/identifier: " hostname
        if [[ -n "$hostname" ]] && validate_hostname "$hostname"; then
            break
        else
            echo "Error: Invalid hostname format. Please use alphanumeric characters and hyphens only."
        fi
    done

    # Prompt for IP address
    while true; do
        read -p "Enter the IP address: " ip_address
        if validate_ip "$ip_address"; then
            break
        else
            echo "Error: Invalid IP address format. Please enter a valid IPv4 address (e.g., 192.168.1.1)"
        fi
    done
    
    # Collect additional SAN entries
    additional_sans_str=$(collect_san_entries)
    read -ra additional_sans <<< "$additional_sans_str"
    
elif [[ $# -ge 2 ]]; then
    # Non-interactive mode
    hostname="$1"
    ip_address="$2"
    shift 2  # Remove first two arguments
    additional_sans=("$@")  # Remaining arguments are additional SANs
    
    echo "Running in non-interactive mode..."
    echo
    
    # Validate hostname
    if [[ -z "$hostname" ]] || ! validate_hostname "$hostname"; then
        echo "Error: Invalid hostname format '$hostname'. Please use alphanumeric characters and hyphens only."
        exit 1
    fi
    
    # Validate IP address
    if ! validate_ip "$ip_address"; then
        echo "Error: Invalid IP address format '$ip_address'. Please enter a valid IPv4 address."
        exit 1
    fi
    
    # Validate additional SAN entries
    for san_entry in "${additional_sans[@]}"; do
        if ! validate_san_entry "$san_entry"; then
            echo "Error: Invalid SAN entry format '$san_entry'. Use TYPE:value format (e.g., DNS:example.com, IP:192.168.1.1)"
            exit 1
        fi
    done
    
else
    echo "Error: Invalid number of arguments."
    echo
    show_usage
fi

# Construct the full domain name
full_domain="${hostname}.fluxlab.systems"

# Build SAN string
san_string="IP:${ip_address}"
if [[ ${#additional_sans[@]} -gt 0 ]]; then
    for san_entry in "${additional_sans[@]}"; do
        san_string="${san_string},${san_entry}"
    done
fi

echo
echo "=== Configuration Summary ==="
echo "Hostname: $hostname"
echo "Full domain: $full_domain"
echo "Primary IP Address: $ip_address"
if [[ ${#additional_sans[@]} -gt 0 ]]; then
    echo "Additional SAN entries:"
    for san_entry in "${additional_sans[@]}"; do
        echo "  - $san_entry"
    done
fi
echo "Complete SAN string: $san_string"
echo

# Only prompt for confirmation in interactive mode
if [[ $# -eq 0 ]]; then
    read -p "Proceed with certificate generation? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "Certificate generation cancelled."
        exit 0
    fi
else
    echo "Proceeding with certificate generation..."
fi

echo
echo "=== Starting Certificate Generation ==="

# Change to CA directory
echo "Changing to CA directory..."
cd /root/ca

# List directory contents for verification
echo "Current directory contents:"
ls -al

echo
echo "Generating private key..."
# Generate private key
openssl genrsa -out "intermediate/private/${full_domain}.key.pem" 2048

# Set secure permissions on private key
echo "Setting permissions on private key..."
chmod 400 "intermediate/private/${full_domain}.key.pem"

echo "Creating certificate signing request..."
# Create certificate signing request with IP SAN and additional SANs
openssl req -config intermediate/openssl.conf \
    -addext "subjectAltName = ${san_string}" \
    -key "intermediate/private/${full_domain}.key.pem" \
    -new -sha256 \
    -out "intermediate/csr/${full_domain}.csr.pem"

echo "Signing certificate..."
# Sign the certificate
openssl ca -config intermediate/openssl.conf \
    -extensions server_cert \
    -days 375 \
    -notext \
    -md sha256 \
    -in "intermediate/csr/${full_domain}.csr.pem" \
    -out "intermediate/certs/${full_domain}.cert.pem"

echo "Copying certificate to output directory..."
# Copy certificate to output directory
cp "intermediate/certs/${full_domain}.cert.pem" /var/ssl/out/

echo
echo "=== Certificate Generation Complete ==="
echo
echo "Files generated:"
echo "  Private Key: intermediate/private/${full_domain}.key.pem"
echo "  Certificate: intermediate/certs/${full_domain}.cert.pem"
echo "  Output Copy: /var/ssl/out/${full_domain}.cert.pem"
echo
echo "Certificate details:"
echo "  Common Name: ${full_domain}"
echo "  Subject Alternative Names: ${san_string}"
echo

echo "=== Private Key Content ==="
cat "intermediate/private/${full_domain}.key.pem"

echo
echo "=== Certificate Content ==="
cat "/var/ssl/out/${full_domain}.cert.pem"

echo
echo "Certificate generation completed successfully!"
echo "The certificate includes the following Subject Alternative Names:"
echo "  ${san_string//,/, }"
echo "Remember to keep the private key secure and never share it."
