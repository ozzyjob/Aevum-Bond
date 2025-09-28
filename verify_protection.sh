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
