<script setup>
import PublicShell from '../../Components/PublicShell.vue';
import SeoHead from '../../Components/SeoHead.vue';
import LegalDoc from '../../Components/LegalDoc.vue';
defineOptions({ layout: null });
defineProps({ updated: { type: String, default: '' } });
</script>

<template>
    <PublicShell>
        <SeoHead title="Terms &amp; Conditions" description="TradePilot Terms &amp; Conditions — the rules for using our automated crypto trading software." path="/terms" />
        <LegalDoc title="Terms &amp; Conditions" :updated="updated" summary="The rules for using TradePilot. Short version: it’s software, you stay in control of your account, and you carry the trading risk.">

            <div class="legal">
                <p>These Terms &amp; Conditions (“Terms”) govern your access to and use of the TradePilot website, applications, and services (collectively, the “Service”), operated by <strong>TradePilot</strong> (“TradePilot”, “we”, “us”), based in India. By creating an account or using the Service, you agree to these Terms. If you do not agree, do not use the Service.</p>

                <h2>1. Eligibility</h2>
                <p>You must be at least 18 years old and legally able to enter into a binding contract. You may not use the Service where doing so would violate any applicable law, regulation, or sanctions, or where automated or crypto trading is restricted. You are solely responsible for determining whether your use of the Service is lawful in your jurisdiction.</p>

                <h2>2. What TradePilot Is — and Is Not</h2>
                <p>TradePilot is <strong>software</strong> that lets you configure and run automated trading strategies on a brokerage or exchange account that <strong>you</strong> own and connect. TradePilot is <strong>not</strong> a broker-dealer, exchange, money transmitter, custodian, investment adviser, financial planner, or fiduciary. We do not hold your funds, do not manage assets on your behalf, and do not provide personalized investment, financial, legal, or tax advice. All trading decisions and configurations are yours.</p>

                <h2>3. Accounts &amp; Security</h2>
                <p>You are responsible for maintaining the confidentiality of your login credentials and for all activity under your account. Notify us immediately of any unauthorized use. You agree to provide accurate information and to keep it current. We may suspend or terminate accounts that violate these Terms or pose a security or legal risk.</p>

                <h2>4. Connecting Brokers &amp; Third-Party Accounts</h2>
                <p>To trade live, you connect your own third-party broker or exchange account using API credentials. Your use of those third parties is governed by their own terms. You authorize TradePilot to transmit orders to your connected account based on your configured strategies. You are responsible for ensuring your API keys have the correct (and not excessive) permissions, and for any costs, fees, or margin/settlement obligations imposed by your broker.</p>

                <h2>5. No Investment Advice; No Guarantee</h2>
                <p>Nothing in the Service constitutes investment, financial, legal, or tax advice or a recommendation to buy, sell, or hold any asset. Strategy names, signals, AI outputs, backtests, and performance metrics are informational only. <strong>We do not guarantee any profit, outcome, accuracy, or performance.</strong> Past performance and simulated or backtested results are not indicative of future results. See our <a href="/risk-disclosure">Risk Disclosure</a>.</p>

                <h2>6. Automated Trading &amp; Software Risks</h2>
                <p>Automated trading carries unique risks, including software errors, misconfiguration, latency, connectivity or exchange outages, data inaccuracies, slippage, partial fills, and rapid market moves. Orders may execute, fail, or repeat in ways you did not intend. You accept these risks and agree to monitor your account, use available safeguards (e.g., paper mode, stops, kill-switch, position limits), and remain responsible for all resulting trades.</p>

                <h2>7. Subscriptions, Fees &amp; Billing</h2>
                <p>Certain features require a paid subscription. Fees, billing cycles, and any trial terms are presented at purchase. Unless stated otherwise or required by law, fees are non-refundable. We may change pricing prospectively with notice. You are responsible for all applicable taxes and for any third-party broker/exchange fees, which are separate from TradePilot’s fees.</p>

                <h2>8. Acceptable Use</h2>
                <p>You agree not to: (a) use the Service for unlawful, fraudulent, or manipulative trading (e.g., market manipulation, wash trading); (b) violate any exchange, broker, or market rules; (c) reverse engineer, scrape, overload, or interfere with the Service; (d) access the Service via unauthorized means or share access; or (e) use the Service to harm others or infringe their rights.</p>

                <h2>9. Third-Party Services &amp; AI</h2>
                <p>The Service integrates third-party providers (e.g., brokers/exchanges, market-data feeds, AI/LLM providers, hosting, and analytics). We are not responsible for third-party acts, outages, or content. AI features provide context and may veto or score trades; they do not predict prices and may be inaccurate. You use third-party integrations at your own risk and under their terms.</p>

                <h2>10. Intellectual Property</h2>
                <p>The Service, including its software, design, and content, is owned by TradePilot or its licensors and protected by law. We grant you a limited, non-exclusive, non-transferable, revocable license to use the Service for its intended purpose. You retain ownership of your own data and configurations.</p>

                <h2>11. Disclaimers</h2>
                <p>THE SERVICE IS PROVIDED “AS IS” AND “AS AVAILABLE” WITHOUT WARRANTIES OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, ACCURACY, AND NON-INFRINGEMENT. WE DO NOT WARRANT THAT THE SERVICE WILL BE UNINTERRUPTED, ERROR-FREE, SECURE, OR THAT ANY STRATEGY WILL BE PROFITABLE.</p>

                <h2>12. Limitation of Liability</h2>
                <p>TO THE MAXIMUM EXTENT PERMITTED BY LAW, TRADEPILOT AND ITS AFFILIATES, OFFICERS, AND AGENTS WILL NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR EXEMPLARY DAMAGES, OR FOR ANY LOST PROFITS, LOST DATA, OR TRADING LOSSES, ARISING FROM OR RELATED TO THE SERVICE. OUR TOTAL AGGREGATE LIABILITY FOR ANY CLAIM WILL NOT EXCEED THE GREATER OF THE AMOUNTS YOU PAID US IN THE 3 MONTHS BEFORE THE CLAIM OR USD 100. Some jurisdictions do not allow certain limitations; in those cases the limitations apply to the fullest extent permitted.</p>

                <h2>13. Indemnification</h2>
                <p>You agree to indemnify and hold harmless TradePilot from any claims, losses, liabilities, and expenses (including reasonable legal fees) arising out of your use of the Service, your trading activity, your breach of these Terms, or your violation of any law or third-party right.</p>

                <h2>14. Termination</h2>
                <p>You may stop using the Service and close your account at any time. We may suspend or terminate your access at any time for any reason, including violation of these Terms or legal/security risk. Upon termination, your right to use the Service ends; provisions that by their nature should survive (e.g., disclaimers, liability limits, indemnity) will survive.</p>

                <h2>15. Changes to the Service or Terms</h2>
                <p>We may modify the Service or these Terms. Material changes will be posted here with an updated date and, where appropriate, additional notice. Continued use after changes take effect constitutes acceptance.</p>

                <h2>16. Governing Law &amp; Disputes</h2>
                <p>These Terms are governed by and construed in accordance with the laws of <strong>India</strong>, without regard to conflict-of-laws rules. Subject to the arbitration provision below, the courts of India shall have exclusive jurisdiction over any matter arising from these Terms.</p>
                <p><strong>Arbitration.</strong> Any dispute, controversy, or claim arising out of or relating to these Terms, or the breach, termination, or validity thereof, that cannot be resolved amicably shall be referred to and finally settled by arbitration under the <strong>Arbitration and Conciliation Act, 1996</strong> (as amended), before a sole arbitrator appointed by mutual agreement of the parties. The seat and venue of arbitration shall be in India, and the proceedings shall be conducted in English. The arbitral award shall be final and binding on the parties. Nothing in this clause prevents either party from seeking interim or injunctive relief from a court of competent jurisdiction in India.</p>

                <h2>17. Contact</h2>
                <p>Questions about these Terms: <a href="mailto:legal@tradepilot.devifyo.cloud">legal@tradepilot.devifyo.cloud</a>.</p>
            </div>
        </LegalDoc>
    </PublicShell>
</template>

