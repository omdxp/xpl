<program name="conditional" include="math.xpl" version="1.0">
  <description>Conditional example</description>
  <function name="main">
    <body>
      <assign var="x">
        <call function="subtract">
          <param>10</param>
          <param>5</param>
        </call>
      </assign>
      <if>
        <condition>
          <call function="subtract">
            <param>x</param>
            <param>5</param>
          </call>
        </condition>
        <then>
          <print>"x minus 5 is non-zero"</print>
        </then>
        <else>
          <print>"x minus 5 is zero"</print>
        </else>
      </if>
    </body>
  </function>
</program>
