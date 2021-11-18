@verbatim
<!doctype html>
<html>
<head>
    <title>redis-demo</title>
    <link rel="stylesheet" href="https://cdn.materialdesignicons.com/5.3.45/css/materialdesignicons.min.css">
    <link rel="stylesheet" href="https://unpkg.com/buefy/dist/buefy.min.css">
</head>
<body>
<div id="app">
    <chatty></chatty>
</div>
<script src="https://unpkg.com/vue@2.6.12/dist/vue.js"></script>
<script src="https://unpkg.com/buefy/dist/buefy.min.js"></script>
<script type="type/x-template" id="chatty">
    <section class="hero is-fullheight">
        <div class="hero-head">
            <header class="hero is-primary is-bold">
                <div class="hero-body">
                    <div class="container">
                        <p class="title">
                            redis-demo
                        </p>
                    </div>
                </div>
            </header>
        </div>

        <div class="hero-body">
            <div style="height: 50vh; width: 100%; overflow-y: auto; overflow-x: hidden" ref="msgdiv">
                <p v-for="message in messages" :style="{ padding: '.25em', textAlign: 'right', overflowWrap: 'normal' }">
                    <span :class="{tag: true, 'is-medium': true}">{{ message.channel }}: {{ message.message }}</span>
                </p>
            </div>
        </div>
    </section>
</script>
<script defer>
    Vue.component('chatty', {
        template: '#chatty',
        data() {
            return {
                msgClass: true,
                userInput: '',
                messages: [],
                isSmallTalk: false,
            };
        },
        computed: {
        },
        created() {
            const socket = new WebSocket('ws://localhost:9001');
            socket.addEventListener('message', ({data}) => {
                const result = JSON.parse(data);
                this.messages.push(result);
                console.log(data);
                this.$nextTick(() => {
                    let msgdiv = this.$refs.msgdiv;
                    msgdiv.scrollTop = msgdiv.scrollHeight;
                });
            });
        },
    });

    new Vue({
        el: '#app',
    });
</script>
</body>
</html>
@endverbatim
