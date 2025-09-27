#!/bin/bash

# 🛡️ Simplified Branch Protection Setup
# Manual configuration with GitHub CLI

echo "🛡️ CONFIGURING BRANCH PROTECTION - SIMPLIFIED APPROACH"
echo "======================================================"
echo ""

REPO="ozzyjob/Aevum-Bond"
BRANCH="main"

echo "📋 Repository: $REPO"
echo "🌿 Branch: $BRANCH"
echo ""

# Step 1: Basic branch protection
echo "🔧 Step 1: Enabling basic branch protection..."
gh api repos/$REPO/branches/$BRANCH/protection \
    --method PUT \
    --raw-field required_status_checks='null' \
    --raw-field enforce_admins=true \
    --raw-field required_pull_request_reviews='null' \
    --raw-field restrictions='null'

echo "✅ Basic protection enabled"
echo ""

# Step 2: Enable pull request reviews
echo "🔧 Step 2: Configuring pull request reviews..."
gh api repos/$REPO/branches/$BRANCH/protection/required_pull_request_reviews \
    --method PATCH \
    --field required_approving_review_count=1 \
    --field dismiss_stale_reviews=true \
    --field require_code_owner_reviews=true

echo "✅ Pull request reviews configured"
echo ""

# Step 3: Configure status checks
echo "🔧 Step 3: Setting up required status checks..."
gh api repos/$REPO/branches/$BRANCH/protection/required_status_checks \
    --method PATCH \
    --field strict=true \
    --field contexts='["test","security","coverage"]'

echo "✅ Status checks configured"
echo ""

# Step 4: Additional restrictions
echo "🔧 Step 4: Applying additional restrictions..."
gh api repos/$REPO/branches/$BRANCH/protection \
    --method PATCH \
    --field allow_force_pushes=false \
    --field allow_deletions=false

echo "✅ Force push and deletion protection enabled"
echo ""

# Verify final status
echo "🔍 VERIFYING PROTECTION STATUS..."
echo "================================"
gh api repos/$REPO/branches/$BRANCH/protection --jq '.url' 2>/dev/null && echo "✅ Branch protection is ACTIVE" || echo "⚠️ Manual configuration needed"

echo ""
echo "🎯 MANUAL CONFIGURATION STEPS (if needed):"
echo "=========================================="
echo "1. Go to: https://github.com/$REPO/settings/branches"
echo "2. Click 'Add rule' or edit existing rule"
echo "3. Branch name pattern: main"
echo "4. Enable these settings:"
echo "   ✅ Require a pull request before merging"
echo "   ✅ Require approvals (1)"
echo "   ✅ Dismiss stale PR reviews when new commits are pushed"  
echo "   ✅ Require review from CODEOWNERS"
echo "   ✅ Require status checks to pass before merging"
echo "   ✅ Require branches to be up to date before merging"
echo "   ✅ Restrict pushes that create files larger than 100MB"
echo "   ✅ Include administrators"
echo ""
echo "📊 Required status checks to add:"
echo "   - test"
echo "   - security" 
echo "   - coverage"
echo "   - performance"
echo ""
echo "🚀 After configuration, your branch will be fully protected!"
