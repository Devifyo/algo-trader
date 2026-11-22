<script setup>
import PublicShell from '../../Components/PublicShell.vue';
import SeoHead from '../../Components/SeoHead.vue';
import LegalDoc from '../../Components/LegalDoc.vue';
defineOptions({ layout: null });
defineProps({ updated: { type: String, default: '' } });
</script>

<template>
    <PublicShell>
        <SeoHead title="Privacy Policy" description="How TradePilot collects, uses and protects your data — aligned with India's DPDP Act 2023, GDPR and CCPA." path="/privacy" />
        <LegalDoc title="Privacy Policy" :updated="updated" summary="What we collect, why, who we share it with, and the rights you have over your data. Plain version: we don’t sell it, and your exchange keys never leave the vault.">

            <div class="legal">
                <p>This Privacy Policy explains how <strong>TradePilot</strong> (“TradePilot”, “we”, “us”), based in India, collects, uses, shares, and protects personal information when you use the TradePilot website and services (the “Service”). By using the Service, you agree to this Policy.</p>

                <h2>1. Information We Collect</h2>
                <p><strong>Account &amp; profile:</strong> name, email, organization, password (stored hashed), and your selected risk profile and preferences.</p>
                <p><strong>Broker/exchange credentials:</strong> API keys you choose to connect. These are treated as secrets, stored encrypted in a dedicated secrets vault, and used only to place and manage orders you have configured. We do not store them in plain text and do not use them for any other purpose.</p>
                <p><strong>Trading &amp; usage data:</strong> bot configurations, strategies, orders, fills, positions, logs, and performance metrics generated through your use.</p>
                <p><strong>Technical data:</strong> IP address, device/browser information, timestamps, and similar logs for security and operation.</p>
                <p><strong>Cookies:</strong> session and preference cookies needed to keep you signed in and remember settings (see Section 9).</p>
                <p><strong>Payment data:</strong> if you subscribe, billing is handled by a third-party payment processor; we do not store full card numbers.</p>

                <h2>2. How We Use Information</h2>
                <p>We use information to: operate and secure the Service; authenticate you; execute and monitor your configured trades; provide analytics, logs, and performance reporting; provide support; process payments; comply with legal obligations; detect and prevent fraud or abuse; and improve the Service.</p>

                <h2>3. Legal Bases &amp; Indian Data Protection (DPDP Act, 2023)</h2>
                <p>We process personal data to perform our contract with you, for our legitimate uses, to comply with legal obligations, and/or with your consent where required. As we are based in India, we handle personal data in accordance with the <strong>Digital Personal Data Protection Act, 2023 (DPDP Act)</strong> and applicable rules — including processing for lawful purposes with notice and consent, and honoring your rights as a Data Principal (Section 10). For users outside India, additional laws such as the GDPR or CCPA may also apply. You may withdraw consent at any time without affecting prior processing.</p>

                <h2>4. How We Share Information</h2>
                <p><strong>Brokers/exchanges:</strong> order and account instructions are transmitted to the broker you connect.</p>
                <p><strong>Service providers:</strong> hosting/infrastructure, market-data feeds, AI/LLM providers (for market-context features), analytics, logging, and payment processors — bound by confidentiality and processing only as needed.</p>
                <p><strong>Legal &amp; safety:</strong> when required by law, regulation, legal process, or to protect rights, safety, and security.</p>
                <p><strong>Business transfers:</strong> in connection with a merger, acquisition, or asset sale, subject to this Policy.</p>
                <p>We do <strong>not</strong> sell your personal information.</p>

                <h2>5. AI Features &amp; Your Data</h2>
                <p>Some features send limited, relevant context (e.g., market or news text, anonymized signals) to third-party AI providers to score sentiment/regime or veto trades. We aim to minimize what is shared and do not send your broker secrets to AI providers. AI outputs are informational and may be inaccurate.</p>

                <h2>6. Data Retention</h2>
                <p>We retain personal information for as long as your account is active and as needed to provide the Service, comply with legal/tax/audit obligations, resolve disputes, and enforce agreements. Trading records may be retained longer where required by law. You may request deletion as described below; some data may be retained as legally permitted or required.</p>

                <h2>7. Security</h2>
                <p>We use technical and organizational measures including encryption of secrets, access controls, network isolation, tenant data segregation, and audit logging. No system is perfectly secure; we cannot guarantee absolute security. Protect your credentials and use strong, unique passwords.</p>

                <h2>8. International Transfers</h2>
                <p>Your information may be processed in countries other than your own. Where required, we use appropriate safeguards (e.g., standard contractual clauses) for cross-border transfers.</p>

                <h2>9. Cookies</h2>
                <p>We use strictly necessary cookies for authentication and session management, and preference cookies (e.g., theme). You can control cookies through your browser, but disabling necessary cookies may break sign-in and core features.</p>

                <h2>10. Your Rights</h2>
                <p>Depending on your location, you may have rights to access, correct, update, and erase your personal data, to nominate another person to exercise rights in case of death or incapacity, and to grievance redressal (under India’s DPDP Act, 2023); to restrict or port your data and object to certain processing (GDPR); and (for California residents) to know what is collected and to opt out of “sale”/“sharing” (we do not sell). To exercise rights or raise a grievance, contact us below; we may verify your identity before responding.</p>

                <h2>11. Children</h2>
                <p>The Service is not directed to anyone under 18, and we do not knowingly collect data from children. If you believe a child provided us data, contact us and we will delete it.</p>

                <h2>12. Changes to This Policy</h2>
                <p>We may update this Policy. Material changes will be posted here with a new “Last updated” date and, where appropriate, additional notice. Continued use after changes constitutes acceptance.</p>

                <h2>13. Contact &amp; Grievance Officer</h2>
                <p>Privacy questions, data-rights requests, or grievances: <a href="mailto:privacy@tradepilot.devifyo.cloud">privacy@tradepilot.devifyo.cloud</a>. Data fiduciary: <strong>TradePilot, India</strong>. In line with India’s DPDP Act, 2023, we will acknowledge and address grievances within the timelines prescribed by applicable law.</p>
            </div>
        </LegalDoc>
    </PublicShell>
</template>

