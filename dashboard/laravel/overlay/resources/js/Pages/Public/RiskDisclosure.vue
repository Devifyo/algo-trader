<script setup>
import PublicShell from '../../Components/PublicShell.vue';
import SeoHead from '../../Components/SeoHead.vue';
import LegalDoc from '../../Components/LegalDoc.vue';
defineOptions({ layout: null });
defineProps({ updated: { type: String, default: '' } });
</script>

<template>
    <PublicShell>
        <SeoHead title="Risk Disclosure" description="Trading involves substantial risk. Read TradePilot's full risk disclosure before using automated crypto trading software." path="/risk-disclosure" />
        <LegalDoc title="Risk Disclosure" :updated="updated" summary="Please read this before you connect a real exchange account. Automated trading can lose money — sometimes quickly.">

            <div class="legal-warn"><p style="margin:0">
                <strong>Important:</strong> Trading involves substantial risk. You can lose some or all of your capital. Only trade with money you can afford to lose. TradePilot provides software tools, not financial advice, and does not guarantee any profit or outcome.</p>
            </div>

            <div class="legal">
                <p>This Risk Disclosure describes key risks of using the TradePilot Service to automate trading. It is not exhaustive. By using the Service you acknowledge and accept these risks.</p>

                <h2>1. No Guarantee of Profit</h2>
                <p>No strategy, signal, AI output, or feature guarantees profit or protection from loss. Markets are uncertain. Any expected returns are hypothetical and may never be realized.</p>

                <h2>2. Risk of Loss</h2>
                <p>You may lose part or all of your invested capital. Some instruments and market conditions can cause losses rapidly. You are solely responsible for funds in your connected broker/exchange account.</p>

                <h2>3. Crypto-Asset Volatility</h2>
                <p>Crypto-assets are highly volatile and can move sharply at any hour, including weekends. Liquidity can disappear, spreads can widen, and exchanges can halt trading, delist assets, or experience outages — any of which can cause significant slippage or losses.</p>

                <h2>4. Automated &amp; Algorithmic Trading Risk</h2>
                <p>Automated systems can fail or behave unexpectedly due to software bugs, misconfiguration, incorrect parameters, data errors, latency, network or exchange outages, API changes, or rate limits. Orders may not execute, may execute at unfavorable prices, may partially fill, or may repeat. Protective features (stops, take-profits, trailing stops, kill-switch) may fail to trigger or fill at the intended price, especially in fast or illiquid markets.</p>

                <h2>5. Past &amp; Simulated Performance</h2>
                <p>Past performance and any backtested, simulated, or paper-trading results are <strong>not</strong> indicative of future results. Backtests are subject to assumptions, hindsight bias, and overfitting, and do not fully reflect real costs, slippage, funding, or liquidity. Live results will differ.</p>

                <h2>6. Leverage &amp; Margin</h2>
                <p>If you use leverage or margin at your broker, losses can exceed your initial deposit and you may be subject to liquidation and additional obligations. Understand your broker’s margin, settlement, and liquidation rules before trading.</p>

                <h2>7. Fees, Costs &amp; Taxes</h2>
                <p>Trading incurs broker/exchange fees, spreads, funding, and possibly other costs that reduce returns. You are responsible for any taxes on your trading activity. In <strong>India</strong>, income from the transfer of virtual digital assets (VDAs) is generally taxed at a flat <strong>30%</strong> (plus applicable surcharge and cess), losses from VDAs cannot be set off against other income, and a <strong>1% Tax Deducted at Source (TDS)</strong> may apply on transfers above prescribed thresholds. Tax laws can change. Consult a qualified tax professional.</p>

                <h2>8. Regulatory Uncertainty (India)</h2>
                <p>The legal and regulatory treatment of crypto-assets and automated trading in India is evolving and uncertain. Future laws, regulations, RBI/SEBI guidance, exchange restrictions, or government action could affect your ability to access markets, the availability of the Service, or the value and liquidity of assets. You are responsible for ensuring your use of the Service complies with all laws applicable to you.</p>

                <h2>9. Third-Party Dependencies</h2>
                <p>The Service depends on third parties (brokers/exchanges, data feeds, AI providers, hosting). Their failures, outages, errors, or changes can disrupt the Service and affect your trades. TradePilot does not control and is not responsible for third-party performance.</p>

                <h2>10. Not Investment Advice</h2>
                <p>TradePilot is not a broker-dealer, investment adviser, or fiduciary, and provides no personalized recommendations. Strategy names and AI features are informational only and do not predict prices. You make all trading decisions.</p>

                <h2>11. Your Responsibility</h2>
                <p>You are responsible for configuring strategies appropriately, setting risk limits, monitoring your account, testing in paper mode before going live, and deciding whether any activity is suitable for your circumstances and lawful in your jurisdiction. Consider seeking advice from an independent, licensed financial professional before trading.</p>
            </div>
        </LegalDoc>
    </PublicShell>
</template>

