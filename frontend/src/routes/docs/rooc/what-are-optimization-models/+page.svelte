<script lang="ts">
    import Page from '$cmp/layout/Page.svelte';
    import SyntaxHighlighter from "$cmp/SyntaxHighlighter.svelte";
    import Card from "$cmp/layout/Card.svelte";
</script>

<svelte:head>
    <title>
        Rooc docs - What are optimization models
    </title>
    <meta name="description" content="Discover the basics of optimization models. Learn what decision variables, objective functions, and constraints, demonstrating can be used to formulate and solve optimization problems using the ROOC syntax."/>
</svelte:head>

<Page cropped padding='1rem' mobilePadding='1rem' gap="1rem">
    <h1>
        What is optimization?
    </h1>
    <p>
        Imagine you have a $50 store coupon for groceries. For your diet you need to buy certain types of food – like
        fruits,
        vegetables, and dairy –. Ideally, you'd spend exactly $50. However, it
        might not always be possible to spend the exact amount. <br/>
        So, your goal is to buy the required groceries and spend
        as little extra money as possible beyond the coupon's value. <br/>
        This is a simple example of what we call
        optimization.
    </p>
    <p>
        Optimization is all about finding the best solution to a problem, given certain limitations or requirements. The
        "best" solution depends on what you're trying to achieve. In our grocery example, the "best" solution means
        spending as close to $50 as possible without spending too much more. We want to minimize the amount we have to
        pay out of pocket.
    </p>
    <p>
        Let's say you just randomly pick items: 3 apples ($12), 2 carrots ($4), and 1 milk ($5). That's $21 total.
        You're far from using your full coupon! You could buy more, but what should you buy to get closer to $50 without
        spending a lot more than $50? Trying every possible combination of fruits, vegetables, and dairy could take a
        long time.
    </p>
    <h1>The Components of an Optimization Problem</h1>

    <p>
        Our grocery shopping scenario, though simple, contains all the essential elements of an optimization problem,
        which
        we will call a <em>model</em>.
        Let's break it down:
    </p>
    <h2>
        Decision Variables
    </h2>
    <p>
        These are the things we can <em>choose</em> or <em>control</em>. In our
        case, the decision variables are <em>how many of each item we buy</em>. We can represent these with
        variables:
    </p>
    <ul>
        <li><em>a</em> = number of apples</li>
        <li><em>b</em> = number of bananas</li>
        <li><em>ca</em> = number of carrots</li>
        <li><em>s</em> = number of spinach</li>
        <li><em>m</em> = number of milk</li>
        <li><em>ch</em> = number of cheese</li>
    </ul>
    <p>
        For example, <em>a</em> = 3 means we buy 3 apples. These variables are what we'll be changing to find the
        best solution.
    </p>

    <h2>Objective Function</h2>
    <p>
        This is what we're trying to <em>minimize</em> (we can also maximise). In our case, we want to
        <em>minimize</em> the amount of money we spend <em>beyond</em> our $50 coupon. We can express this
        mathematically:
    </p>
    <p>Total Cost = 4<em>a</em> + 3<em>b</em> + 2<em>ca</em> + 3<em>s</em> + 5<em>m</em> + 6<em>ch</em></p>
    <p>Extra Spending = Total Cost - 50</p>
    <p>Our objective function is to minimize the "Extra Spending".</p>

    <h2>Constraints </h2>
    <p>
        These are the <em>limitations</em> or <em>rules</em> we must follow. We say that our model
        is <em>Subject to (s.t.)</em> those constraints. <br/>
        In our grocery example, we have a few constraints:
    </p>
    <ul>
        <li><strong>Minimum Fruit:</strong> <em>a</em> + <em>b</em> ≥ 3 (We must buy at least 3 fruits)</li>
        <li><strong>Minimum Vegetables:</strong> <em>ca</em> + <em>s</em> ≥ 2 (We must buy at least 2
            vegetables)
        </li>
        <li><strong>Minimum Dairy:</strong> <em>m</em> + <em>ch</em> ≥ 1 (We must buy at least 1 dairy product)
        </li>
        <li><strong>Minimum Spending:</strong> 4<em>a</em> + 3<em>b</em> + 2<em>ca</em> + 3<em>s</em> +
            5<em>m</em> + 6<em>ch</em> ≥ 50 (We must spend at least 50)
        </li>
        <li><strong>Non-negative quantities:</strong> <em>a</em>, <em>b</em>, <em>ca</em>, <em>s</em>,
            <em>m</em>, <em>ch</em> ≥ 0 (We can't buy a negative number of items)
        </li>
    </ul>
    <p>
        By defining these three components-decision variables, objective function, and constraints—we've turned our
        everyday grocery problem into a formal optimization model that can be solved using mathematical
        techniques.
    </p>
    <p>
        Let's put everything together in a model using the ROOC syntax:
    </p>
    <Card padding="1rem">
        <SyntaxHighlighter
                language="rooc"
                source={`min (4a + 3b + 2ca + 3s + 5m + 6ch) - 50
subject to
    a + b >= 3
    ca + s >= 2
    m + ch >= 1
    4a + 3b + 2ca + 3s + 5m + 6ch >= 50
define
    a, b, ca, s, m, ch as IntegerRange(0, 100)`}
        />
    </Card>
    <p>
        And using the solvers inside rooc, we are able to find a solution with cost <em>0</em>, so we managed to spend
        exactly $50 <br/>
        The food we picked are: <em>apples = 10</em>, <em>carrots = 2</em>, <em>cheese = 1</em>
    </p>
    <h1>
        Why is optimization useful?
    </h1>

    <p>
        Optimization helps us make better
        decisions when we face limited resources, competing goals, or complex systems. <br/>
        Here are some other examples:
    </p>
    <ul>
        <li>
            <strong>Logistics and Transportation:</strong> Companies like delivery services and airlines use
            optimization to plan routes, schedule deliveries, and manage fleets. The goal might be to minimize fuel
            consumption, delivery times, or overall costs.
        </li>
        <li>
            <strong>Manufacturing and Production:</strong> In manufacturing, optimization is used to determine the best
            production schedules, manage inventory levels, and design efficient production lines. The goal might be to
            maximize production output, minimize waste, or reduce production costs. For example, a factory might use
            optimization to determine the optimal mix of products to manufacture given limited resources like raw
            materials and machine time.
        </li>
        <li>
            <strong>Finance and Investment:</strong> Financial institutions use optimization to build investment
            portfolios that maximize returns while minimizing risk. Optimization techniques can help determine the best
            allocation of assets across different investment classes.
        </li>
        <li>
            <strong>Energy and Utilities:</strong> Power companies use optimization to manage the distribution of
            electricity, optimize energy production, and schedule maintenance. The goal might be to minimize energy
            losses, reduce costs, or ensure a reliable power supply.
        </li>
    </ul>

    <p>
        These are just a few examples of how optimization is used across diverse fields. In essence, any situation where
        you're trying to make the "best" decision given certain constraints can potentially benefit from optimization
        techniques. By formulating the problem mathematically and using appropriate algorithms, we can find solutions
        that are proven to be the best.
    </p>

</Page>


<style>
    @import "../common.scss";

    li {
        margin: 0.5rem 0;
    }
</style>