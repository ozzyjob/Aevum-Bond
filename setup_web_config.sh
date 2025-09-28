#!/bin/bash

# 🛡️ Direct Web Configuration Script
# Opens the correct GitHub pages for manual setup

echo "🛡️ AEVUM & BOND - DIRECT WEB CONFIGURATION"
echo "=========================================="
echo ""

REPO="ozzyjob/Aevum-Bond"
SETTINGS_URL="https://github.com/$REPO/settings/branches"

echo "📋 Repository: $REPO"
echo "🔗 Settings URL: $SETTINGS_URL"
echo ""

# Check if browser commands are available
if command -v xdg-open >/dev/null 2>&1; then
    BROWSER_CMD="xdg-open"
elif command -v open >/dev/null 2>&1; then
    BROWSER_CMD="open"
elif command -v firefox >/dev/null 2>&1; then
    BROWSER_CMD="firefox"
elif command -v google-chrome >/dev/null 2>&1; then
    BROWSER_CMD="google-chrome"
else
    BROWSER_CMD=""
fi

echo "🌐 OPENING GITHUB SETTINGS IN BROWSER..."
echo "========================================"

if [ -n "$BROWSER_CMD" ]; then
    echo "✅ Opening: $SETTINGS_URL"
    $BROWSER_CMD "$SETTINGS_URL" &
    sleep 2
    echo "✅ Browser opened successfully!"
else
    echo "❌ No browser command found"
    echo "👆 Please manually open: $SETTINGS_URL"
fi

echo ""
echo "📋 MANUAL CONFIGURATION CHECKLIST:"
echo "=================================="
echo ""
echo "1. ✅ Click 'Add rule' button"
echo "2. ✅ Branch name pattern: main"
echo "3. ✅ Enable 'Require a pull request before merging'"
echo "4. ✅ Set 'Required number of reviewers: 1'"
echo "5. ✅ Enable 'Dismiss stale PR reviews when new commits are pushed'"
echo "6. ✅ Enable 'Require review from CODEOWNERS'"
echo "7. ✅ Enable 'Require status checks to pass before merging'"
echo "8. ✅ Enable 'Require branches to be up to date before merging'"
echo "9. ✅ Add status checks: test, security, coverage, performance"
echo "10. ✅ Enable 'Restrict pushes that create files larger than 100MB'"
echo "11. ✅ Enable 'Include administrators'"
echo "12. ✅ Click 'Create' to save"
echo ""

# Create a quick verification script
echo "🔍 CREATING VERIFICATION SCRIPT..."
cat > verify_protection.sh << 'EOF'
#!/bin/bash
echo "🔍 VERIFYING BRANCH PROTECTION STATUS"
echo "===================================="
echo ""

# Check protection status via GitHub API
gh api repos/ozzyjob/Aevum-Bond/branches/main/protection 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ BRANCH PROTECTION: ACTIVE"
    echo ""
    echo "🛡️ Protection Details:"
    gh api repos/ozzyjob/Aevum-Bond/branches/main/protection --jq '{
        "required_pull_request_reviews": .required_pull_request_reviews.required_approving_review_count,
        "required_status_checks": .required_status_checks.contexts,
        "enforce_admins": .enforce_admins.enabled,
        "allow_force_pushes": .allow_force_pushes.enabled,
        "allow_deletions": .allow_deletions.enabled
    }'
else
    echo "❌ BRANCH PROTECTION: NOT CONFIGURED"
    echo ""
    echo "⚠️  Please complete manual configuration at:"
    echo "   https://github.com/ozzyjob/Aevum-Bond/settings/branches"
fi

echo ""
echo "🎯 Next: Create test PR to verify workflow"
echo "gh pr create --title 'Test Protection' --body 'Testing branch protection setup'"
EOF

chmod +x verify_protection.sh
echo "✅ Created: verify_protection.sh"
echo ""

echo "⏰ TIMING:"
echo "========="
echo "⏱️  Configuration time: 2-3 minutes"
echo "🎯 Verification: Run ./verify_protection.sh after setup"
echo ""

echo "🤖 GITHUB COPILOT AGENTS READY:"
echo "==============================="
echo "After configuration, your agents will:"
echo "✅ Create PRs automatically"
echo "✅ Require @ozzyjob review (CODEOWNERS)"
echo "✅ Run all 58 tests before merge"
echo "✅ Follow security audit process"
echo "✅ Maintain code quality standards"
echo ""

echo "🎉 READY TO CONFIGURE!"
echo "====================="
echo "👆 Browser should be open, follow checklist above"
echo "🔍 Run ./verify_protection.sh when done"
echo ""
